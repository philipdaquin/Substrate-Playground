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
}

	
	