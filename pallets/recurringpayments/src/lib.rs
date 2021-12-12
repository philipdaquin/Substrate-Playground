// This file is part of Gamma Finance.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//* Subscription Model based Payment System: */
//	- Connect and Role based system -> Allow users to rate the merchants 
//	- use proxy pallet to force user to pay else -> cancel subscription 	

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::too_many_arguments)]
#![warn(unused_imports)]

use sp_runtime::MultiSignature;

use sp_runtime::traits::IdentifyAccount;

use sp_runtime::traits::Verify;

use sp_arithmetic::Percent;

use sp_std::{convert::TryInto, boxed::Box};

//use substrate_fixed::types::*;

use codec::{Decode, Encode, HasCompact};
use frame_support::{pallet_prelude::*, ensure, storage::child, PalletId,

	traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons, UnixTime,
	
	fungibles::{Inspect, Mutate, Transfer
	
	}},
	
	sp_runtime::{traits::{AccountIdConversion, AtLeast32BitUnsigned, Saturating, Zero, Hash, AtLeast32Bit}, ArithmeticError,
	
	sp_std::prelude::*
	}
};
	
use frame_support::{storage::ChildTriePrefixIterator, traits::schedule::{Named as ScheduleNamed, DispatchTime}, dispatch::Dispatchable};
use frame_system::ensure_root;

use sp_runtime::{
	traits::{CheckedSub, One}, Permill};

use scale_info::TypeInfo;
use frame_system::{pallet_prelude::OriginFor, ensure_signed};
use sp_runtime::traits::StaticLookup;

mod builder;
mod types;
use crate::types::*;
mod functions;
//use crate::functions::*;
//pub use pallet_scheduler::pallet::*;
pub use pallet::*;


#[frame_support::pallet]
pub mod pallet {
use super::*;	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_proxy::Config + pallet_scheduler::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		
		//	The currency that this fund accepts
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		//	'PalletId' for the Subscription Pallet 
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		
		// Type used for expressing timestamp.
		type Moment: AtLeast32Bit + Parameter + Default + Copy + From<u64>;
		//	Dispatchable		
		type Info: Parameter + Dispatchable<Origin = <Self as frame_system::Config>::Origin> + From<Call<Self>>;

		// The origin which may forcible create or destroy a payment 
		//	The origin that executes dispatchable for user recurring payment 
		type ForceOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

		/// The maximum length of a name or symbol stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		//	The amount to be held on deposit by the depositor of a Payment Plan 
		//	the create call will be used to create a new fund, a deposit must be made first 
		type SubmissionDeposit: Get<BalanceOf<Self>>;

		/// The Scheduler.
		type Scheduler: ScheduleNamed<Self::BlockNumber, Self::Info, <Self as pallet::Config>::PalletsOrigin>;

		// Overarching type of all pallets origins.
		type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;


	}
	pub type PaymentIndex = u32;
	pub type SubscriptionIndex = u32; 
	//pub type AccountId<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountId<T>>>::Balance;
	pub type AccountId<T> = <T as frame_system::Config>::AccountId;
	pub type BlockNumber<T> = <T as frame_system::Config>::BlockNumber;
	
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn paymentid)]
	pub type PaymentId<T> = StorageValue<_, PaymentIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn subscribe)]
	pub type Subscriptions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		AccountId<T>, 
		Subscription<AccountId<T>, BlockNumber<T>, BalanceOf<T>>,
		ValueQuery
	>;
	#[pallet::storage]
	#[pallet::getter(fn subscription_id)]
	pub type SubscriptionId<T> = StorageValue<_, SubscriptionIndex, ValueQuery>; 

	#[pallet::storage]
	#[pallet::getter(fn info)]
	pub type PaymentInfo<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		PaymentIndex,
		PaymentPlan<AccountId<T>, BalanceOf<T>, BoundedVec<u8, T::StringLimit>>,
		ValueQuery
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PaymentPlanCreated { 
			merchant: T::AccountId,
			id: PaymentIndex, 
			now: T::BlockNumber,
		},
		SubcriptionCreated {
			subscriber: T::AccountId, 
			id: PaymentIndex,
			now: T::BlockNumber
		},
		SubcriptionCancelled { 
			subcriber: T::AccountId, 
			id: PaymentIndex, 
			now: T::BlockNumber
		},
		PaymentSent { 
			from: T::AccountId, 
			to: T::AccountId, 
			amount: BalanceOf<T>,
			id: PaymentIndex,
			now: T::BlockNumber
		},
		PaymentCollected { 
			merchant: T::AccountId, 
			id: PaymentIndex,
			now: T::BlockNumber
		},
		PaymentPlanKilled { 
			merchant: Option<T::AccountId>, 
			id: PaymentIndex, 
			now: T::BlockNumber
		},
		PaymentRefundedToUser { 
			user: T::AccountId, 
			id: PaymentIndex, 
			now: T::BlockNumber
		},
		RecurringPaymentCancelled { 
			user: T::AccountId,
			id: PaymentIndex,
			now: T::BlockNumber
		},
		PaymentPlanDestroyed { 
			merchant: T::AccountId,
			id: PaymentIndex,
			now: T::BlockNumber,
		},
		EditedPaymentPlan { 
			merchant: T::AccountId,
			id: PaymentIndex, 
			now: T::BlockNumber
		},
		PaymentRefundedToMerchant { 
			merchant: T::AccountId,
			id: PaymentIndex, 
			now: T::BlockNumber,
		},
		TransferApproved { 
			owner: T::AccountId, 
			new_owner: T::AccountId,
			id: PaymentIndex,
			now: T::BlockNumber
		},
		Frozen { 
			freezer: T::AccountId, 
			id: PaymentIndex,
			now: T::BlockNumber
		},
		UnFrozen { 
			who: Option<T::AccountId>, 
			id: PaymentIndex,
			now: T::BlockNumber
		},
		RefundedUsers { 
			sender: T::AccountId,
			id: PaymentIndex,
			now: T::BlockNumber, 
		},
		PartiallyRefunded { 
			sender: T::AccountId,
			id: PaymentIndex, 
			now: T::BlockNumber,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		//	Triggered when Users are trying to generate duplicate id of another existign id
		PaymentPlanAlreadyExist,
		//	Triggered when Users are trying to call functions on a Non Existing Id
		PaymentPlanDoesNotExist,
		//	The singing account 
		InsufficientBalance,
		//	Unpriviledge call
		UnAuthorisedCall,
		//	Invalud metadata given 
		BadMetadata,
		//	Storage Error
		Unknown,
		//	The Payment plan in Frozen 
		Frozen,
		//	user trying to freeze an already frozen payment 
		AlreadyUnFrozen,
		//	User trying to execute payments on non existing users
		UserDoesNotExist,
		//	Users trying to execute payments on non subscribed users 
		NotASubscriber,
		//	Zero subscribers 
		NoSubscribersFound,
		
		UserNotSubscribed,
		//	Users trying to execute calls on non due days
		NotDueYet,
		//	Payment required cant be set to zero 
		MinCannotBeZero,
		// Invalid Required Payment 
		InvalidPayment,
		//	Failed to calculate the ratio to return to the user 
		FailedToCalculateRatio,
		//	No Refund 

	}

	///	Additional Builder methods is Also Provided as follows: 
	///	let new_payment = Self::new_payment_plan()
			// 	.identified_by(payment_id.clone())
			// 	.owned_by(merchant.clone())
			// 	.with_name(bounded_name.clone())
			// 	.min_payment(required_payment)
			// 	.new_deposit(Zero::zero())
			// 	.payment_frequency(frequency)
			// 	.total_subscribers(Zero::zero())
			// 	.freezer(merchant)
			// 	.freeze_payments(false) 
			// 	.build();
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_payment_system(
			origin: OriginFor<T>, 
			required_payment: BalanceOf<T>,
			user_frequency: Frequency,
			name: Vec<u8>,
			freezer: Option<<T::Lookup as StaticLookup>::Source>,
			schedule_periodic_collection: Option<Frequency>,
		//	collection_portion: Option<Portion>,
		) -> DispatchResult { 
			let merchant = ensure_signed(origin)?;
			//	ensure merchants can only create payment plans
			let payment_id = Self::next_payment_id()?;
			//	Now in Seconds
			let now = frame_system::Pallet::<T>::block_number();
			
			Self::do_create(
				payment_id, 
				name,
				required_payment,
				user_frequency,
				merchant.clone(), 
				freezer,
				schedule_periodic_collection,
				//collection_portion,
				T::SubmissionDeposit::get(),
				Event::PaymentPlanCreated { 
					merchant,
					id: payment_id, 
					now,
				}
			)
		}
		///	Subscribe to Payment Function 
		/// This allows the user to automate recurring payments according to their desired merchant 
		/// 
		/// 1. User Sends the Min Payment for the Desired Payment Plan (based on the payment id)
		/// 	The user has the ability to set the parameters on how many times they want to stay subscribed to
		/// 	this merchant (as indicated on 'num_frequency') 
		/// 2. The call checks if the Payment Plan does no exist, call Err 'PaymentPlanDoesNotExist' if true, else proceed  
		/// 3. Check if the min_payment is greater than the required payment of the payment_plan
		/// 4. Transfer Money into the Merchant's Fund
		/// 5. Increase Total Deposits inside the Paymetn Plan Struct 
		/// 6. Update Storage PaymentInfo
		/// 7  Insert User's payment into the Child-Trie of the Merchant's Fund, then update the Child-Trie
		/// 8. Create a Subcription Info for the User and a Subscription Id (we generate a new one per call) to ensure we structure our onchain storage properly
		/// 9. Get the Date today, calculate the next payment (which is specified on Types as BlockNumber)
		/// 10. Register the root as the Delagator for the user, so we can force payments later on if the dispatchable fails to work (Users shoudl be aware of this)
		/// 11. Register a Scheduled call to allow for next_payments to occur automatically, based on the Frequency inside of PaymentPlan 
		/// 12. Emit event 
		/// 
		/// Methods Provided for Subscription Struct 
		/// let subcription_info = Self::new_subscription()
		/// .account_id(subscriber.clone())
		/// .start_date(start_date)
		/// .next_payment(next_payment)
		///.frequency_type(payment_plan.frequency)
		/// .set_num_freq(num_frequency)
		/// .min_payments(min_payment)
		/// .subscribed_list(sub_list)
		/// .build();
	 
		#[pallet::weight(10_000)]
		pub fn subscribe_payment(
			origin: OriginFor<T>,
			payment_id: PaymentIndex,
			min_payment: BalanceOf<T>,
			num_frequency: Option<u32>
		) -> DispatchResult{ 
			let subscriber = ensure_signed(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			Self::do_subscribed( 
				payment_id,
				min_payment,
				num_frequency,
				subscriber.clone(), 
				Event::PaymentSent { 
					from: subscriber.clone(),
					to: Self::fund_account_id(payment_id),
					amount: min_payment.clone(),
					id: payment_id.clone(),
					now
				},
				Event::SubcriptionCreated { 
					subscriber: subscriber.clone(),
					id: payment_id,
					now
				}
			)
		}
		//	Ensure the merchants can only call this function, block off unauthorised transactions 
		//	Allow for merchants to withdraw a portion of funds 
		// Event::PaymentCollected { 
		// 	merchant,
		// 	id: payment_id,
		// 	now: T::Moment::now()
		// }
		#[pallet::weight(10_000)]
		pub fn collect_payments(
			origin: OriginFor<T>,
			payment_id: PaymentIndex,
		//	specified_portion: Option<Portion>, // later to be implemented using Substrate Fixed
			schedule_periodic_collection: Option<Frequency>
		) -> DispatchResult { 
			let merchant = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();
			
			Self::do_collect_payments(
				merchant.clone(),
				payment_id, 
			//	specified_portion,
				Event::PaymentCollected { 
					merchant: merchant.clone(),
					id: payment_id,
					now,
				}
			)
		}
		///	Cancel User Subscriber
		/// To ensure fair service, we will refund a portion of the user's unspent period of their subscription
		///	 
		///
		#[pallet::weight(10_000)]
		pub fn cancel_subscription(
			origin: OriginFor<T>,
			payment_id: PaymentIndex,
		) -> DispatchResult { 
			let subscriber = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();
			Self::do_cancel(
				subscriber.clone(),
				payment_id,
				Event::PaymentRefundedToUser { 
					user: subscriber.clone(),
					id: payment_id, 
					now
				},
				Event::RecurringPaymentCancelled { 
					user: subscriber.clone(),
					id: payment_id,
					now
				}
			)
		}
		//	ensure the user can edit this while subscribers do not lose their positions 
		// 	if editing, unreserve the funds inside the trie into the depositor immediately but do not destroy the fund 
		#[pallet::weight(10_000)]
		pub fn edit_payment_plan(
			origin: OriginFor<T>,
			id: PaymentIndex,
			new_payment: BalanceOf<T>,
			frequency: Frequency,
			name_s: Vec<u8>,
			freezer: Option<<T::Lookup as StaticLookup>::Source>,
			schedule_periodic_collection: Option<Frequency>,
		//	collection_portion: Option<Portion>,
		) -> DispatchResult { 
			let seller = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();
			Self::do_edit_plan(
				id, 
				name_s,
				new_payment,
				frequency,
				seller.clone(),
				freezer,
				schedule_periodic_collection,
			//	collection_portion,
				Event::EditedPaymentPlan { 
					merchant: seller.clone(), 
					id, 
					now
				},
			)			
		}
		//	If merchant is present, or not, refund the remaining balances to the merchant owner found 
		//	inside the struct 
		#[pallet::weight(10_000)]
		pub fn force_delete_plan(
			origin: OriginFor<T>,
			payment_id: PaymentIndex
		) -> DispatchResult { 
			let maybe_owner: Option<AccountId<T>> = match T::ForceOrigin::try_origin(origin) { 
				Ok(_) => None, 
				Err(origin) => Some(ensure_signed(origin)?),
			};
			let now = frame_system::Pallet::<T>::block_number();
			Self::do_delete(
				maybe_owner.clone(),
				payment_id, 
				Event::PaymentPlanKilled { 
					merchant: maybe_owner.clone(),
					id: payment_id, 
					now
				}
			)
		}
		///	Transfer ownership of a payment plan to an approved delegate 
		///	Origin 
		#[pallet::weight(10_000)]
		pub fn transfer_ownership_plan(
			origin: OriginFor<T>,
			payment_id: PaymentIndex,
			delegate: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult { 
			let maybe_owner: Option<AccountId<T>> = match T::ForceOrigin::try_origin(origin) { 
				Ok(_) => None, 
				Err(origin) => Some(ensure_signed(origin)?)
			};
			let delegate = T::Lookup::lookup(delegate)?;
			let now = frame_system::Pallet::<T>::block_number();
			Self::do_transfer_ownership(
				maybe_owner.clone(), 
				payment_id, 
				delegate.clone(),
				Event::TransferApproved { 
					owner: maybe_owner.unwrap().clone(), 
					new_owner: delegate.clone(),
					id: payment_id,
					now
				}
			)
		}
		///	Issue a new class of payment plans from a priviledged origin 
		/// There will be no assets stored inside of this
		///	Required Payment Will Be Set to Zero 
		#[pallet::weight(10_000)]
		pub fn force_create_new(
			origin: OriginFor<T>,
			owner: <T::Lookup as StaticLookup>::Source,
			name: Vec<u8>,
			frequency: Frequency,
			required_payment: BalanceOf<T>,
			freezer: Option<<T::Lookup as StaticLookup>::Source>,
			set_frozen: bool,
			schedule_periodic_collection: Option<Frequency>,
		//	collection_portion: Option<Portion>,
		) -> DispatchResult { 
			T::ForceOrigin::ensure_origin(origin)?;	
			
			let owner = T::Lookup::lookup(owner)?;
			let payment_id = Self::next_payment_id()?;
			let now = frame_system::Pallet::<T>::block_number();
			
			Self::do_create(
				payment_id, 
				name,
				required_payment,
				frequency,
				owner.clone(), 
				freezer,
				schedule_periodic_collection,
			//	collection_portion,
				T::SubmissionDeposit::get(),
				Event::PaymentPlanCreated { 
					merchant: owner,
					id: payment_id, 
					now
				}
			)
		}
		///	Block further unpriviledge transfers from an account 
		/// Origin must be Signed and the sender should be the Freezer of the Payment PLan 
		#[pallet::weight(10_000)]
		pub fn freeze_payment_plan(
			origin: OriginFor<T>,
			payment_id: PaymentIndex,
		) -> DispatchResult {
			let maybe_owner: Option<AccountId<T>> = match T::ForceOrigin::try_origin(origin) { 
				Ok(_) => None, 
				Err(origin) => Some(ensure_signed(origin)?),
			};
			let now = frame_system::Pallet::<T>::block_number();
			Self::do_freeze(
				maybe_owner.clone(),
				payment_id,
				Event::Frozen { 
					freezer: maybe_owner.unwrap().clone(),
					id: payment_id, 
					now
				}
			)
		}
		#[pallet::weight(10_000)]
		pub fn unfreeze_payment_plan(
			origin: OriginFor<T>,
			payment_id: PaymentIndex
		) -> DispatchResult { 
			let maybe_owner = match T::ForceOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)? ),
			};
			let now = frame_system::Pallet::<T>::block_number();
			Self::do_unfreeze(
				maybe_owner.clone(),
				payment_id.clone(),
				Event::UnFrozen { 
					who: maybe_owner.clone(),
					id: payment_id.clone(), 
					now
				}
			)
		}
		///	Remove User from the Subscription List 
		/// Refund user for any unspent subscription period 
		/// Remove proxy from user to prevent scheduled dispatchables 
		/// Destroy User Subscription 
		#[pallet::weight(10_000)]
		pub fn force_cancel_subscription(
			origin: OriginFor<T>,
			payment_id: PaymentIndex,
			subscriber: <T::Lookup as StaticLookup>::Source
		) -> DispatchResult { 
			T::ForceOrigin::ensure_origin(origin)?;
			let subscriber = T::Lookup::lookup(subscriber)?;
			let now = frame_system::Pallet::<T>::block_number();
			Self::do_cancel(
				subscriber.clone(),
				payment_id.clone(),
				Event::PaymentRefundedToUser { 
					user: subscriber.clone(),
					id: payment_id.clone(), 
					now
				},
				Event::RecurringPaymentCancelled { 
					user: subscriber.clone(),
					id: payment_id.clone(),
					now
				}
			)
		}
		// Force user to payment without refunding 
		#[pallet::weight(10_000)]
		pub fn force_payment(
			origin: OriginFor<T>,
			subscriber: <T::Lookup as StaticLookup>::Source,
			payment_id: PaymentIndex,
			required_payment: BalanceOf<T>,
		) -> DispatchResult { 
			T::ForceOrigin::ensure_origin(origin)?;
			//let subscriber = T::Lookup::lookup(subscriber)?;
			let now = frame_system::Pallet::<T>::block_number();		
			let user = T::Lookup::lookup(subscriber)?;		
			Self::do_force_pay(
				user.clone(),
				payment_id, 
				required_payment,
				Event::RecurringPaymentCancelled { 
					user: user.clone(),
					id: payment_id.clone(),
					now
				},
				Event::PaymentSent { 
					from: user.clone(),
					to: Self::fund_account_id(payment_id),
					amount: required_payment,
					id: payment_id.clone(),
					now
				}
			)
		}
		//	Refund first then generate new values 
		#[pallet::weight(10_000)]
		pub fn force_clear_payment_plan(
			origin: OriginFor<T>,
        	payment_id: PaymentIndex, 
        	name: Vec<u8>,
        	new_owner: <T::Lookup as StaticLookup>::Source,
        	min_balance: BalanceOf<T>,
        	frequency: Frequency,
        	freezer: Option<<T::Lookup as StaticLookup>::Source>,
        	is_frozen: bool,
        	schedule_periodic_collection: Option<Frequency>,
		) -> DispatchResult { 
			let maybe_owner = match T::ForceOrigin::try_origin(origin) {
					Ok(_) => None,
					Err(origin) => Some(ensure_signed(origin)? 
				),
			};
			let now = frame_system::Pallet::<T>::block_number();
			Self::refund_to_users(maybe_owner.clone(), payment_id)?;
			//	Set Values to User input
			Self::force_default(
				maybe_owner.clone(),
				payment_id, 
				name,
				new_owner,
				min_balance,
				frequency,
				freezer,
				is_frozen,
				schedule_periodic_collection,
				Event::EditedPaymentPlan { 
					merchant: maybe_owner.unwrap(), 
					id: payment_id, 
					now
				},
			)
		}
	} 	
	
	impl<T: Config> Pallet<T> { 
			
		//	Does the id already exist in our storage? 
		// pub(crate) fn verify_new_plan(id: &PaymentIndex) -> bool { 
		//     PaymentInfo::<T>::contains_key(id)
		// }

		//	Create a new payment plan using PaymentPlanBuilder
		// pub(super) fn new_payment_plan() -> PaymentPlanBuilder<T::AccountId, T::Balance> { 
		//     PaymentPlanBuilder::<T::AccountId, T::Balance>::default()
		// }

		//	Create subscription to a Payment Plan 
		// pub(super) fn new_subscription() -> SubscriptionBuilder<T::AccountId, T::Moment, T::Balance> { 
		//     SubscriptionBuilder::<T::AccountId, T::Moment, T::Balance>::default()
		// }

		//	This is where recurring payments are paid into 
		pub(super) fn fund_account_id(idx: PaymentIndex) -> T::AccountId { 
			T::PalletId::get().into_sub_account(idx)
		}
		//	Track Payment Index
		pub(super) fn next_payment_id() -> Result<u32, DispatchError> {
			PaymentId::<T>::try_mutate(|index| -> Result<u32, DispatchError> {
				let current_id = *index;
				*index = index.checked_add(1).ok_or(ArithmeticError::Overflow)?;
				Ok(current_id)
			})
		}
		pub(super) fn next_subscriber_id() -> Result<u32, DispatchError> { 
			SubscriptionId::<T>::try_mutate(|id| -> Result<u32, DispatchError> { 
				let curr = *id;
				*id = id.checked_add(1).ok_or(ArithmeticError::Overflow)?;
				Ok(curr)
			})
		}
		pub(super) fn trie_iterator(id: &PaymentIndex) -> ChildTriePrefixIterator<(T::AccountId, (BalanceOf<T>, Vec<u8>))> { 
			ChildTriePrefixIterator::<_>::with_prefix_over_key::<Identity>(
				&Self::id_from_index(id),
				&[],
			)
		}
		// 	Function to find the id associated with the fund id (child trie)
		//	Each fund stores information about it ***contributors and their ***contributions in a child trie 
		
		//	This helper function calculates the id of the associate child trie 
		pub(super) fn id_from_index(
			index: &PaymentIndex
		) -> child::ChildInfo { 
			let mut buf = Vec::new();
			buf.extend_from_slice(b"payment");
			buf.extend_from_slice(&index.to_le_bytes()[..]);

			child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
		}
		//	Put Payment under a key: user account 
		pub(super) fn insert_payment(
			index: PaymentIndex, 
			who: &T::AccountId, 
			balance: &BalanceOf<T>
		) {
			let id = Self::id_from_index(&index);
			who.using_encoded(|b| child::put(&id, b, &balance));
		}
		//	Get the value paid by the user 
		pub(super) fn get_payment_info(
			index: PaymentIndex, 
			who: &T::AccountId
		) -> BalanceOf<T> {
			let id = Self::id_from_index(&index);
			who.using_encoded(|b| child::get_or_default::<BalanceOf<T>>(&id, b))
		}
		pub(super) fn kill_paymentsystem(index: PaymentIndex) {
			let id = Self::id_from_index(&index);
			// The None here means we aren't setting a limit to how many keys to delete.
			// Limiting can be useful, but is beyond the scope of this recipe. For more info, see
			// https://crates.parity.io/frame_support/storage/child/fn.kill_storage.html
			child::kill_storage(&id, None);
		}
		pub(super) fn calculate_next_duedate(
			start: T::BlockNumber, 
			frequency: Frequency,
		) -> Result<T::BlockNumber, DispatchError> {
			let now = frame_system::Pallet::<T>::block_number();
			let freq: u32 = frequency.frequency();
			let mut due_date = start.saturating_add(T::BlockNumber::from(freq));
			
			if due_date == now { 
				due_date = now.saturating_add(T::BlockNumber::from(freq))
			}
			//  Return the due_date for BlockNumber 
			Ok(due_date)
		}
	
		pub(super) fn do_create(
			payment_id: PaymentIndex,
			name: Vec<u8>,
			required_payment: BalanceOf<T>,
			frequency: Frequency,
			merchant: AccountId<T>,
			freezer: Option<<T::Lookup as StaticLookup>::Source>,
			schedule_periodic_collection: Option<Frequency>,
		//	collection_portion: Option<Portion>,
			deposit: BalanceOf<T>,
			event: Event<T>,
		) -> DispatchResult {
					
			ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanAlreadyExist);
			let bounded_name: BoundedVec<u8, T::StringLimit> =
				name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
			let freezer = freezer.map(T::Lookup::lookup).transpose()?;
			
			PaymentInfo::<T>::insert(
				payment_id,
				PaymentPlan::<AccountId<T>, BalanceOf<T>, BoundedVec<u8, T::StringLimit>> { 
						merchant: merchant.clone(),
						name: bounded_name.clone(),
						payment_id,
						required_payment,
						total_deposits: Zero::zero(),
						frequency,
						num_subscribers: Zero::zero(),
						freezer,
						is_frozen: false,
						schedule_periodic_collection: schedule_periodic_collection.clone()
				}
			);
			
			let imbalance = <T as pallet::Config>::Currency::withdraw(
				&merchant, 
				deposit, 
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath
			)?;
			//	Create a fund, imbalance is empty 
			<T as pallet::Config>::Currency::resolve_creating(
				&Self::fund_account_id(payment_id),
				imbalance
			);
			let now = frame_system::Pallet::<T>::block_number();
			
			//  Schedule Periodic Collections
			//	TODO: 'Change maybe_period'
			T::Scheduler::schedule_named(
				bounded_name.to_vec(),
				DispatchTime::At(now),
				None,
				63,
				frame_system::RawOrigin::Root.into(),
				Call::collect_payments {
					payment_id, 
					schedule_periodic_collection: schedule_periodic_collection.clone(), 
			//		specified_portion: collection_portion,
				}.into()
			);

			Self::deposit_event(event);
			Ok(())

		}
		pub(super) fn do_collect_payments(
			merchant: AccountId<T>, 
			payment_id: PaymentIndex,
		//	specified_portion: Option<Portion>,
			event: Event<T>,
		) -> DispatchResult {
			ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
			
			let mut payment_info = PaymentInfo::<T>::get(&payment_id);
			
			ensure!(merchant == payment_info.merchant, Error::<T>::UnAuthorisedCall);
			ensure!(!payment_info.total_deposits.is_zero(), Error::<T>::InsufficientBalance);

			// let total_balance = payment_info.total_deposits;
			// //  only collect 50% * total deposit = update the storage 
			// if let Some(amount) = specified_portion { 
			// 	let mut total = payment_info.total_deposits;
			// 	let per_deposit = amount.portion().saturating_mul(total); 

			// 	let mut user_requested_amount: BalanceOf<T> = total.saturating_mul(per_deposit.into());
			// 	let mut new_total_deposit_in_storage = total.checked_sub(&user_requested_amount).ok_or(ArithmeticError::Underflow)?;

			// 	payment_info.total_deposits = new_total_deposit_in_storage;

			// 	PaymentInfo::<T>::insert(payment_id, payment_info);
			// 	 //	Ensure we are not chargin fees when the user decides to collect their payments
			// 	}
			// }

			let _ = <T as pallet::Config>::Currency::resolve_creating( 
				&payment_info.merchant, 
				<T as pallet::Config>::Currency::withdraw(
					&Self::fund_account_id(payment_id),
					payment_info.total_deposits,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
					)?
			);
			PaymentInfo::<T>::insert(payment_id, payment_info);

			Self::deposit_event(event);
			
			Ok(())
		}
		pub(super) fn do_transfer_ownership(
			maybe_owner: Option<AccountId<T>>,
			payment_id: PaymentIndex, 
			delegate: AccountId<T>,
			event: Event<T>, 
		) -> DispatchResult {
			ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
			
			let mut payment_info = PaymentInfo::<T>::get(&payment_id);

			if let Some(check_owner) = maybe_owner { 
				ensure!(payment_info.merchant == check_owner, Error::<T>::UnAuthorisedCall);
			}
			payment_info.merchant = delegate; 
			PaymentInfo::<T>::insert(&payment_id, &payment_info);
			
			Self::deposit_event(event);

			Ok(())
		}
		
		pub(super) fn do_subscribed(
			payment_id: PaymentIndex, 
			min_payment: BalanceOf<T>,
			num_frequency: Option<u32>,
			subscriber: AccountId<T>,
			event_payment_sent: Event<T>,
			event_sub_created: Event<T>,
		) -> DispatchResult {
			
			ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
			
			//	Access Payment plan information  
			let mut payment_plan = PaymentInfo::<T>::get(&payment_id);
			// ensure the user has enough to supplement for required_payment
			ensure!(min_payment >= payment_plan.required_payment, Error::<T>::InsufficientBalance); 
			//	check if the PaymentPlan is Frozen
			ensure!(!payment_plan.is_frozen, Error::<T>::Frozen);
			//	else -> Transfer funds into fund_index 
			<T as pallet::Config>::Currency::transfer(
				&subscriber, 
				&Self::fund_account_id(payment_id),
				min_payment,
				ExistenceRequirement::AllowDeath
			)?;
			payment_plan.total_deposits += min_payment;
			//	Increment Number of Subscribers 
			payment_plan.num_subscribers.checked_add(1).ok_or(ArithmeticError::Overflow)?;
			
			let payment = Self::get_payment_info(payment_id, &subscriber);
			
			let payment = payment.saturating_add(min_payment);
			Self::insert_payment(payment_id, &subscriber, &payment);

			let starting_block = frame_system::Pallet::<T>::block_number();
			let mut due_date = Self::calculate_next_duedate(starting_block, payment_plan.frequency.clone())?;
			let mut sub_list: Vec<u32> = vec![];
			
			sub_list.push(payment_id);
			
			let subscriber_id = Self::next_subscriber_id()?;
			//	Update Storage Subscriptions 
			Subscriptions::<T>::insert(
				subscriber.clone(),
				Subscription { 
					id: subscriber_id,
					owner: subscriber.clone(),
					start: starting_block,
					required_payment: min_payment,
					next_payment: due_date,
					frequency_of: payment_plan.frequency.clone(),
					num_frequency: None, 
					subscribed_to: sub_list.clone() 					
				}
			);
			PaymentInfo::<T>::insert(payment_id, &payment_plan);
			Self::deposit_event(event_payment_sent);
		
			//	Force Payments Takes in Abstract Accounts that are subscribed to payments 
			//	We convert the Subscriber's account back into source and create a create a scheduled dispatchable fucntion  
			let subscriber =  T::Lookup::unlookup(subscriber);
			//	Schedule a dispatchable function based on frequency
			//	Schedule name is based on the PaymentPlan Name
			//	This is specified according to the users preference 
			//* Needs to be Reviewed */
			T::Scheduler::schedule_named(
				payment_plan.name.to_vec(),
				DispatchTime::At(due_date),
				None, 
				Default::default(),
				frame_system::RawOrigin::Root.into(),
				Call::force_payment { 
					subscriber,
					payment_id,
					required_payment: min_payment
				}.into()
			);
			
			Self::deposit_event(event_sub_created);

			Ok(())
		}
		pub(super) fn do_cancel(
			subscriber: AccountId<T>,
			payment_id: PaymentIndex,
			event_refund: Event<T>,
			event_cancelled: Event<T>,
		) -> DispatchResult {
			ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
			//	Get the users contribution 
			let balance = Self::get_payment_info(payment_id, &subscriber);
			let subscriber_info = Subscriptions::<T>::get(&subscriber);
			let now = frame_system::Pallet::<T>::block_number();
			let begin_period = subscriber_info.start;
			let payment_due = subscriber_info.next_payment;
			
			let amount_refunded = Self::calculate_ratio_start_end(
				begin_period,
				payment_due,
				now, 
				balance
			);

			if amount_refunded.is_err() { 
				let _ = <T as pallet::Config>::Currency::resolve_into_existing(&subscriber, 
					<T as pallet::Config>::Currency::withdraw(
					&Self::fund_account_id(payment_id), 
						amount_refunded.unwrap(), 
						WithdrawReasons::TRANSFER, 
						ExistenceRequirement::AllowDeath
					)?
				);
				
				Self::deposit_event(event_refund);
			}
			//	if we get a balance, then we refund it back to the user 
			
			//	Remove Proxy inside the user 
			pallet_proxy::Pallet::<T>::remove_proxies(frame_system::RawOrigin::Root.into());
			//	Remove schedule dispatchable
			let payment_info = PaymentInfo::<T>::get(payment_id);
			
			pallet_scheduler::Pallet::<T>::cancel_named(frame_system::RawOrigin::Root.into(), 
				payment_info.name.to_vec())?;
			
			Self::deposit_event(event_cancelled);
			
			Ok(())
		} 
		pub(super) fn do_edit_plan(
			id: PaymentIndex,
			name_s: Vec<u8>,
			new_payment: BalanceOf<T>,
			frequency: Frequency,
			seller: AccountId<T>,
			freezer: Option<<T::Lookup as StaticLookup>::Source>,
			schedule_periodic_collection: Option<Frequency>,
		//	collection_portion: Option<Portion>,
			event: Event<T>,
		) -> DispatchResult { 
			
			ensure!(!PaymentInfo::<T>::contains_key(&id), Error::<T>::PaymentPlanDoesNotExist);
			//	Access Payment Plan Details 
			let mut payment_info = PaymentInfo::<T>::get(id);
			ensure!(payment_info.merchant == seller, Error::<T>::UnAuthorisedCall);

			let new_name: BoundedVec<u8, T::StringLimit> =
				name_s.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
			let freezer = freezer.map(T::Lookup::lookup).transpose()?;
			
			PaymentInfo::<T>::insert(
				id,
				PaymentPlan::<AccountId<T>, BalanceOf<T>, BoundedVec<u8, T::StringLimit>> { 
					merchant: payment_info.merchant,
					name: new_name,
					payment_id: payment_info.payment_id,
					required_payment: new_payment,
					total_deposits: payment_info.total_deposits,
					frequency,
					num_subscribers: payment_info.num_subscribers,
					freezer,
					is_frozen: payment_info.is_frozen,
					schedule_periodic_collection		
				}
			);
			//	TODO Collection Portion 

			Self::deposit_event(event);
			
			Ok(())
		}
		pub(super) fn do_delete(
			maybe_owner: Option<AccountId<T>>,
			payment_id: PaymentIndex,
			event_payment_killed: Event<T>
		) -> DispatchResult { 
			ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
			let now = frame_system::Pallet::<T>::block_number();
			let payment_info = PaymentInfo::<T>::get(&payment_id);
			//	Check if the Some(Merchant) is the Owner
			if let Some(check_owner) = maybe_owner { 
				ensure!(payment_info.merchant == check_owner, Error::<T>::UnAuthorisedCall);
			}
			//	If total deposit is not zero, return to user 
			if !payment_info.total_deposits.is_zero() {
				//	Ensure we are not charging fees when the user decides to collect their payments
				let _ = <T as pallet::Config>::Currency::resolve_creating( 
					&payment_info.merchant, <T as pallet::Config>::Currency::withdraw(
						&Self::fund_account_id(payment_id),
						payment_info.total_deposits,
						WithdrawReasons::TRANSFER,
						ExistenceRequirement::AllowDeath,
					)?
				);
				Self::deposit_event(Event::PaymentRefundedToMerchant { 
					merchant: payment_info.merchant,
					id: payment_id, 
					now
				});
			}
			//	Delete from storage 
			PaymentInfo::<T>::remove(&payment_id);
			T::Scheduler::cancel_named(payment_info.name.to_vec());

			Self::deposit_event(event_payment_killed);	

			Ok(())
		}
		pub(super) fn do_freeze(
			merchant: Option<AccountId<T>>, 
			payment_id: PaymentIndex,
			event_freeze: Event<T>,
		) -> DispatchResult {
			ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);

			let mut payment_info = PaymentInfo::<T>::get(&payment_id);
			
			if let Some(check_owner) = merchant { 
				ensure!(payment_info.freezer == Some(check_owner), Error::<T>::UnAuthorisedCall);
			}
			
			payment_info.is_frozen = true;
			
			PaymentInfo::<T>::insert(&payment_id, payment_info);
			
			Self::deposit_event(event_freeze);        

			Ok(())
		}
		pub(super) fn do_unfreeze(
			maybe_owner: Option<AccountId<T>>,
			payment_id: PaymentIndex,
			event_unfreeze: Event<T>,
		) -> DispatchResult { 
			let mut payment_info = PaymentInfo::<T>::get(&payment_id);
			ensure!(!payment_info.is_frozen, Error::<T>::AlreadyUnFrozen);

			if let Some(check_owner) = maybe_owner { 
				ensure!(payment_info.freezer == Some(check_owner), Error::<T>::UnAuthorisedCall);
			}
			payment_info.is_frozen = true;
			
			PaymentInfo::<T>::insert(&payment_id, payment_info);

			Self::deposit_event(event_unfreeze);
			Ok(())
		} 
		pub(super) fn do_force_cancel(
			subscriber: AccountId<T>, 
			payment_id: PaymentIndex,
		) -> DispatchResult {
			
			let now = frame_system::Pallet::<T>::block_number();
			Self::do_cancel(
				subscriber.clone(),
				payment_id,
				Event::PaymentRefundedToUser { 
					user: subscriber.clone(),
					id: payment_id, 
					now,
				},
				Event::RecurringPaymentCancelled { 
					user: subscriber.clone(),
					id: payment_id,
					now,
				}
			)
		}
		pub(super) fn do_force_pay(
			subscriber: AccountId<T>,
			payment_id: PaymentIndex,
			required_payment: BalanceOf<T>,
			event_cancelled: Event<T>,
			event_paid: Event<T>,
		) -> DispatchResult {		
			ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
			
			//let admin = T::ForceOrigin::ensure_origin(admin)?;
			let subscriber = subscriber; 
			let mut payment_info = PaymentInfo::<T>::get(&payment_id);
			let mut subscription_info = Subscriptions::<T>::get(subscriber.clone());
			let now = frame_system::Pallet::<T>::block_number();

			//  Check if the required payments matches with the users subcription 
			ensure!(subscription_info.required_payment == payment_info.required_payment, Error::<T>::InvalidPayment);
			//	Check if the user is associated with the subscription info 
			ensure!(subscription_info.owner == subscriber.clone(), Error::<T>::NotASubscriber);
			//	Check how many users are subscribed to this payment plan, if zero, emit error
			ensure!(!payment_info.num_subscribers.is_zero(), Error::<T>::NoSubscribersFound);
			//	Check if the payment plan is blocking any transfers
			ensure!(!payment_info.is_frozen, Error::<T>::Frozen);
			//	Check if the user is subscribed to the payment plan 
			ensure!(subscription_info.subscribed_to.contains(&payment_id), Error::<T>::UserNotSubscribed);
			//	Check if the user is meant to pay now 
			ensure!(subscription_info.next_payment == now, Error::<T>::NotDueYet);
			//	Get users that are subscribed into this payment plan 
			
			//	Force call the user to transfer funds into the child trie// Root Call
			//	This Assumes the Subscriber has delegated Proxy to the Root 
			//	If the call fails, we remove subscription info, proxy calls, scheduled dispatchables and memberships 
			if Self::transfer_payment(
				frame_system::RawOrigin::Root.into(),  
				T::Lookup::unlookup(subscriber.clone()), 
				payment_id,
				required_payment,
			).is_err() { 
				Subscriptions::<T>::remove(subscriber.clone());
				//	Remove Proxy inside the user 
				pallet_proxy::Pallet::<T>::remove_proxies(frame_system::RawOrigin::Root.into());
				//	Remove schedule dispatchable
				T::Scheduler::cancel_named(payment_info.name.to_vec()).map_err(|_| Error::<T>::PaymentPlanDoesNotExist);
				//  Cancel User Membership
				Self::deposit_event(event_cancelled);
				
			} else { 
				Self::deposit_event(event_paid)

				//  Reschedule next the payment 
			}
			Ok(())
		}
		pub(super) fn force_default(
			maybe_owner: Option<AccountId<T>>,
			payment_id: PaymentIndex, 
			name: Vec<u8>,
			new_owner: <T::Lookup as StaticLookup>::Source,
			min_balance: BalanceOf<T>,
			frequency: Frequency,
			freezer: Option<<T::Lookup as StaticLookup>::Source>,
			is_frozen: bool,
			schedule_periodic_collection: Option<Frequency>,
			event_edit: Event<T>,
		) -> DispatchResult {
			ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
			let mut payment_info = PaymentInfo::<T>::get(&payment_id);
			
			let bounded_name: BoundedVec<u8, T::StringLimit> =
				name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
			let freezer = freezer.map(T::Lookup::lookup).transpose()?;

			PaymentInfo::<T>::try_mutate_exists(payment_id, |info| { 
				let mut curr = info.as_mut().ok_or(Error::<T>::Unknown)?;
				curr.merchant = T::Lookup::lookup(new_owner)?;
				curr.name = bounded_name;
				curr.required_payment = min_balance;
				curr.frequency = frequency;
				curr.num_subscribers = 0;
				curr.freezer = freezer;
				curr.schedule_periodic_collection = schedule_periodic_collection;
				curr.is_frozen = is_frozen;
				
				*info = Some(curr.clone()); 

				Self::deposit_event(event_edit);
				Ok(())
			})
		}
		pub(super) fn refund_to_users(
			maybe_owner: Option<AccountId<T>>,
			payment_id: PaymentIndex,
		) -> DispatchResult { 

			ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
			let mut refunded = 0;
			let mut payment_info = PaymentInfo::<T>::get(payment_id.clone());
			if let Some(check_owner) = maybe_owner.clone() { 
				ensure!(payment_info.merchant == check_owner, Error::<T>::Unknown);
			}
			// Return all user funds 
			let subscriptions = Self::trie_iterator(&payment_id);
			for (user, (balance, _)) in subscriptions { 
				<T as pallet::Config>::Currency::transfer(&Self::fund_account_id(payment_id),
					&user, 
					balance, 
					ExistenceRequirement::AllowDeath,
				)?;
				payment_info.total_deposits = payment_info.total_deposits.saturating_sub(balance);
				refunded += 1;
			}
			PaymentInfo::<T>::insert(&payment_id, payment_info.clone());
			let now = frame_system::Pallet::<T>::block_number();	
			//let sender = maybe_owner.unwrap_or();
			if refunded == payment_info.num_subscribers { 
				Self::deposit_event(Event::RefundedUsers { 
					sender: maybe_owner.unwrap().clone(),
					id: payment_id, 
					now,
				})
			} else { 
				Self::deposit_event(Event::<T>::PartiallyRefunded { 
					sender: maybe_owner.unwrap(),
					id: payment_id, 
					now,
				})
			}
			Ok(())
		}	
		//	Generic Transfer Function 	
		pub(super) fn transfer_payment(
			origin: OriginFor<T>,
			subscriber: <T::Lookup as StaticLookup>::Source,
			payment_id: PaymentIndex,
			min_payment: BalanceOf<T>,
		) -> DispatchResult { 
			ensure_root(origin)?;
			let subscriber = T::Lookup::lookup(subscriber)?;

			ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
			let mut payment_info = PaymentInfo::<T>::get(payment_id);
			// Transfer 
			<T as pallet::Config>::Currency::transfer(
				&subscriber,
				&Self::fund_account_id(payment_id),
				min_payment,
				ExistenceRequirement::AllowDeath
			)?;
			//	Update Storage and Trie
			payment_info.total_deposits += min_payment;
			PaymentInfo::<T>::insert(payment_id, payment_info);
			let new_balance = Self::get_payment_info(payment_id, &subscriber);
			new_balance.saturating_add(min_payment);
			Self::insert_payment(payment_id, &subscriber, &min_payment);
			Ok(())
		}
		//	Calculate the amount to be refunded if the the user wishes to leave early  
		pub(super) fn calculate_ratio_start_end(
			start: T::BlockNumber,
			next_payment: T::BlockNumber,
			now: T::BlockNumber,
			curr_balance: BalanceOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> { 
			ensure!(next_payment != now, Error::<T>::MinCannotBeZero);
			//	Calculate the percent from start/ end 
			//	let ratio = Permill::from_rational(subscription_info.next_payment, subscription_info.start); 
			let elapsed_blocknumber = next_payment - start;
			let elapsed_time_u32: u32 = TryInto::try_into(elapsed_blocknumber).ok().expect("Failed To Convert");

			let remaining_ratio: u32 = 1u32 - elapsed_time_u32;

			Ok(curr_balance * remaining_ratio.into())

		}
	}
}

	
	