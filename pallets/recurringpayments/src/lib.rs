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

//! Functions for the Assets pallet.

#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
use sp_runtime::MultiSignature;
use sp_runtime::traits::IdentifyAccount;
use sp_runtime::traits::Verify;
mod builder;
use crate::builder::*;
use sp_arithmetic::Percent;
use sp_std::convert::TryInto;
use substrate_fixed::types::*;

mod types;
use crate::types::*;

//* Subscription Model based Payment System: */
//	- Connect and Role based system -> Allow users to rate the merchants 
//	- use proxy pallet to force user to pay else -> cancel subscription 	

pub use pallet::*;
use codec::{Decode, Encode, HasCompact};
use frame_support::{pallet_prelude::*, ensure, storage::child, PalletId,
	traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons, UnixTime,
		fungibles::{Inspect, Mutate, Transfer}},
	sp_runtime::{traits::{AccountIdConversion, AtLeast32BitUnsigned, Saturating, Zero, Hash, AtLeast32Bit}, ArithmeticError,
	sp_std::prelude::*
	}
};


use scale_info::TypeInfo;
//	
pub type CurrencyId = u32;
/// Balance of an account.
pub type Balance = u128;
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type PaymentIndex = u32;

#[frame_support::pallet]
pub mod pallet {
	use frame_system::{pallet_prelude::OriginFor, ensure_signed};
use sp_runtime::traits::StaticLookup;

use super::*;
	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_proxy::Config {
		
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		
		//	The currency that this fund accepts
		type Currency: Currency<Self::AccountId>;

		//	'PalletId' for the Subscription Pallet 
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		//	Versatile Assets 
		type Assets: 
		Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
		+ Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
		+ Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
		
		// Type used for expressing timestamp.
		type Moment: AtLeast32Bit + Parameter + Default + Copy + From<u64>;

		//	The units for balance
		type Balance: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;
		
		// The origin which may forcible create or destroy a payment 
		//	The origin that executes dispatchable for user recurring payment 
		type ForceOrigin: EnsureOrigin<Self::Origin>;

		/// The maximum length of a name or symbol stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		//	The amount to be held on deposit by the depositor of a Payment Plan 
		type SubmissionDeposit: Get<BalanceOf<Self>>;
	}

	
	pub type SubscriptionIndex = u32; 
	//pub type AccountId<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOf<T> = <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
	
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn paymentid)]
	pub type PaymentId<T> = StorageValue<_, PaymentIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn subscribe)]
	pub type Subscriptions<T> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, 
		(SubscriptionIndex, Subscription<AccountId, Moment, Balance>),
		ValueQuery
	>;
	#[pallet::storage]
	#[pallet::getter(fn subscription_id)]
	pub type SubscriptionId<T> = StorageValue<_, SubscriptionIndex, ValueQuery>; 

	#[pallet::storage]
	#[pallet::getter(fn info)]
	pub type PaymentInfo<T> = StorageMap<
		_,
		Blake2_128Concat,
		PaymentIndex,
		PaymentPlan<AccountId, Balance>,
		ValueQuery
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PaymentPlanCreated { 
			merchant: T::AccountId,
			id: PaymentIndex, 
			now: T::Moment,
		},
		SubcriptionCreated {
			subscriber: T::AccountId, 
			id: PaymentIndex,
			now: T::Moment
		},
		SubcriptionCancelled { 
			subcriber: T::AccountId, 
			id: PaymentIndex, 
			now: T::Moment
		},
		PaymentSent { 
			from: T::AccountId, 
			to: T::AccountId, 
			amount: T::Balance,
			id: PaymentIndex,
			now: T::Moment
		},
		PaymentCollected { 
			merchant: T::AccountId, 
			id: PaymentIndex,
			now: T::Moment
		},
		PaymentPlanKilled { 
			merchant: T::AccountId, 
			id: PaymentIndex, 
			now: T::Moment 
		},
		PaymentRefundedToUser { 
			user: T::AccountId, 
			id: PaymentIndex, 
			now: T::Moment
		},
		RecurringPaymentCancelled { 
			user: T::AccountId,
			id: PaymentIndex,
			now: T::Moment,
		},
		PaymentPlanDestroyed { 
			merchant: T::AccountId,
			id: PaymentIndex,
			now: T::Moment,
		},
		EditedPaymentPlan { 
			merchant: T::AccountId,
			id: PaymentIndex, 
			now: T::Moment
		},
		PaymentRefundedToMerchant { 
			merchant: T::AccountId,
			id: PaymentIndex, 
			now: T::Moment,
		},
		TransferApproved { 
			owner: T::AccountId, 
			new_owner: T::AccountId,
			id: PaymentIndex,
			now: T::Moment
		},
		Frozen { 
			freezer: T::AccountId, 
			id: PaymentIndex,
			now: T::Moment
		},
		UnFrozen { 
			who: T::AccountId, 
			id: PaymentIndex,
			now: T::Moment
		}

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		PaymentPlanAlreadyExist,
		PaymentPlanDoesNotExist,
		InsufficientBalance,
		UnAuthorisedCall,
		BadMetadata,
		Unknown,
		Frozen,
		AlreadyUnFrozen,
		UserDoesNotExist,
		NotASubscriber,
		NoSubscribersFound,
		UserNotSubscribed,
		NotDueYet

	}
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_payment_system(
			origin: OriginFor<T>, 
			#[pallet::compact] required_payment: T::Balance,
			#[pallet::compact] frequency: Frequency,
			#[pallet::compact] name: Vec<u8>
		) -> DispatchResult { 
			let merchant = ensure_signed(origin)?;
			//	ensure merchants can only create payment plans
			let payment_id = Self::next_payment_id();
			
			match !Self::verify_new_plan(&payment_id) { 
				//	Does not exist, create a new one
				true => { 
					let bounded_name: BoundedVec<u8, T::StringLimit> =
					name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
					
					let deposit = T::SubmissionDeposit::get();
					
					let new_payment = Self::new_payment_plan()
						.identified_by(payment_id.clone())
						.owned_by(merchant.clone())
						.with_name(bounded_name.clone())
						.min_payment(required_payment)
						.new_deposit(Zero::zero())
						.payment_frequency(frequency)
						.total_subscribers(Zero::zero())
						.freezer(merchant)
						.freeze_payments(false)
						.build();

					let imbalance = T::Currency::withdraw(
						&merchant, 
						deposit, 
						WithdrawReasons::TRANSFER,
						ExistenceRequirement::AllowDeath
					)?;
					//	Create a fund, imabalance is empty 
					T::Currency::resolve_creating(
						&Self::fund_account_id(payment_id),
						imbalance
					);
					PaymentInfo::<T>::insert(payment_id, new_payment);
					//	Insert to storage map 
					Self::deposit_event(Event::PaymentPlanCreated { 
						merchant,
						id: payment_id, 
						now: T::Moment::now(),
					});
				},
				_ => { 
					// ALready exists
					return Err(Error::<T>::PaymentPlanAlreadyExist);
				}
			}
			Ok(())
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
		#[pallet::weight(10_000)]
		pub fn subscribe_payment(
			origin: OriginFor<T>,
			#[pallet::compact] payment_id: PaymentIndex,
			#[pallet::compact] min_payment: T::Balance,
			#[pallet::compact] num_frequency: Option<u32>
		) -> DispatchResultWithPostInfo { 
			let subscriber = ensure_signed(origin)?;
			
			match !Self::verify_new_plan(&payment_id) { 
				//	Does not exist 
				true => { 
					return Err(Error::<T>::PaymentPlanDoesNotExist);
				}
				//	Exist
				_ => {
					//	Access Payment plan information  
					let mut payment_plan = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;
					// ensure the user has enough to supplement for required_payment
					ensure!(min_payment >= payment_plan.required_payment, Error::<T>::InsufficientBalance); 
					//	check if the PaymentPlan is Frozen
					ensure!(!payment_plan.is_frozen, Error::<T>::Frozen);
					//	else -> Transfer funds into fund_index 
					T::Currency::transfer(
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

					Self::insert_payment(payment_id, &subscriber, payment);
					//	build Subscription BUilder for user, ensuring the Frequency matches that of the PaymentPlan
					//* -- Below needs revision --  */
					let start_date: Moment = T::Moment::now();
					let next_payment = payment_plan.frequency.frequency()
						.checked_add(start_date)
						.ok_or(ArithmeticError::Overflow)?;
					// Store paymentid into subcription list of user 
					let mut sub_list = vec![];
					
					sub_list.push(payment_id);

					let subcription_info = Self::new_subscription()
						.account_id(subscriber.clone())
						.start_date(start_date)
						.next_payment(next_payment)
						.frequency_type(payment_plan.frequency)
						.set_num_freq(num_frequency)
						.min_payments(min_payment)
						.subscribed_list(sub_list)
						.build();
					
					let subscriber_id = Self::next_subscriber_id();
					
					//	Add Subscriber Index and Subscription Info into Storage
					Subscriptions::<T>::insert(&subscriber, 
						(&subscriber, subcription_info.clone()));
					PaymentInfo::<T>::insert(payment_id, &payment_plan);

					//	Emit Event for Sending Payment to the Merchant 
					Self::deposit_event(Event::PaymentSent { 
						from: &subscriber,
						to: &Self::fund_account_id(payment_id),
						amount: min_payment.clone(),
						id: payment_id.clone(),
						now: T::Moment::now()
					});

					//	Add proxy delegate for the user to allow for scheduled dispatchables
					pallet_proxy::Pallet::<T>::add_proxy(subscriber, 
						T::ForceOrigin, 
						Default::default(), 
						Zero::zero());

					//	Schedule a dispatchable function based on frequency
					//	Schedule name is based on the PaymentPlan Name
					//	This is specified according to the users preference 

					//	Scheduled for the next Payment which involves reducing num_frequency by 1 until it reaches zero
					pallet_scheduler::Pallet::<T>::schedule_named(
						&subscriber,
						payment_plan.name,
						&next_payment,
						Some((payment.frequency, num_frequency.unwrap_or_default())),
						Default::default(),
						Pallet::<T>::subscribe_payment(
							&subscriber,
							&payment_id,
							&min_payment,
							num_frequency.unwrap_or_else(||
								num_frequency.checked_sub(1).ok_or(ArithmeticError::Underflow)
							)
						),
					);
					Self::deposit_event(Event::SubcriptionCreated { 
						subscriber,
						id: payment_id,
						now: T::Moment::now()
					});
				}	
			}
			Ok(().into())
		}
		//	Ensure the merchants can only call this function, block off unauthorised transactions 
		//	Allow for merchants to withdraw a portion of funds 
		#[pallet::weight(10_000)]
		pub fn collect_payments(
			origin: OriginFor<T>,
			#[pallet::compact] payment_id: PaymentIndex,
			#[pallet::compact] specified_portion: Portion, // later to be implemented using Substrate Fixed
			#[pallet::compact] delete_payment_plan_forever: bool,
			#[pallet::compact] periodic_collection: Frequency
		) -> DispatchResult { 
			let merchant = ensure_signed(origin)?;
			
			match !Self::verify_new_plan(&payment_id) { 
				true =>	{ 
					return Err(Error::<T>::PaymentPlanDoesNotExist);
				},
				_ => { 
					let payment_info = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;
					ensure!(merchant == payment_info.merchant, Error::<T>::UnAuthorisedCall);
					ensure!(!payment_info.total_deposits == Zero::zero(), Error::<T>::InsufficientBalance);
					//	Ensure we are not chargin fees when the user decides to collect their payments
					let _ = T::Currency::resolve_creating( 
						&payment_info.merchant, T::Currency::withdraw(
							&Self::fund_account_id(payment_id),
							payment_info.total_deposits,
							WithdrawReasons::TRANSFER,
							ExistenceRequirement::AllowDeath,
						)?
					);
					//	Ensure the user is verified and perform double confirmation 
					if delete_payment_plan_forever { 
						PaymentInfo::<T>::remove(payment_id);
						Self::kill_paymentsystem(payment_id);
						Self::deposit_event(Event::PaymentPlanKilled { 
							merchant,
							id: payment_id, 
							now: T::Moment::now()
						});	
					} else { 
						Self::deposit_event(Event::PaymentCollected { 
							merchant,
							id: payment_id,
							now: T::Moment::now()
						});
					}
					//	Schedule dispatchable for Frequency of Collection

				}
			}
			Ok(())
		}
		///	Cancel User Subscriber
		/// To ensure fair service, we will refund a portion of the user's unspent period of their subscription
		///	 
		///
		#[pallet::weight(10_000)]
		pub fn cancel_subscription(
			origin: OriginFor<T>,
			#[pallet::compact] payment_id: PaymentIndex,
		) -> DispatchResultWithPostInfo { 
			let subscriber = ensure_signed(origin)?;
			
			match !Self::verify_new_plan(&payment_id) { 
				true => { 
					return Err(Error::<T>::PaymentPlanDoesNotExist);
				},
				_ => { 
					//	Get the users contribution 
					let balance = Self::get_payment_info(payment_id, &subscriber);
					let subscriber_info = Subscriptions::<T>::get(&subscriber).ok_or(Error::<T>::Unknown)?;
					
					//	Get User's Subscription information 
					let subscription = subscriber_info.1;
					
					let begin_period = subscription.start;
					let payment_due = subscription.frequency_of.frequency();
					let difference =  payment_due - I32F32::from_num(subscription.start);
					let ratio = I32F32::from_num(difference)/payment_due;
					
					//	How much the user get to keep 
					let new_ratio = I32F32::from_num(1) - ratio;
					
					//	If the ratio is not 1, then we can refund all unused period 
					if new_ratio != I32F32::from_num(1) { 
						//	Refund on current subscription period, ensure the collection has their scheduled date dispatchable function 
						//	The line below is: 
						//	Refunded amount = CurrentPeriodBalance(1 - Ratio between the starting date and due payment date)
						//	ie RefundedToUser = 100(1 - 0.5 <-- or 50% before finishing the period) => refund 50 to user
						let user_refund = balance.saturating_mul(new_ratio);
						
						// Return funds to caller without charging a transfer fee
						let _ = T::Currency::resolve_into_existing(&subscriber, 
							T::Currency::withdraw(
							&Self::fund_account_id(payment_id), 
								user_refund, 
								WithdrawReasons::TRANSFER, 
								ExistenceRequirement::AllowDeath
							)?
						);
						Self::deposit_event(Event::PaymentRefundedToUser { 
							user: subscriber,
							id: payment_id, 
							now: T::Moment::now()
						});
					}
					//	Remove Proxy inside the user 
					pallet_proxy::Pallet::<T>::remove_proxies(&subscriber);
					//	Remove schedule dispatchable
					let payment_info = PaymentInfo::<T>::get(payment_id);
					pallet_scheduler::Pallet::<T>::cancel_named(payment_info.name);
				}
			}
			Self::deposit_event(Event::RecurringPaymentCancelled { 
				user: subscriber,
				id: payment_id,
				now: T::Moment::now(),
			});

			Ok(().into())
		}
		//	ensure the user can edit this while subscribers do not lose their positions 
		// 	if editing, unreserve the funds inside the trie into the depositor immediately but do not destroy the fund 
		#[pallet::weight(10_000)]
		pub fn edit_payment_plan(
			origin: OriginFor<T>,
			#[pallet::compact] id: PaymentIndex,
			#[pallet::compact] new_payment: T::Balance,
			#[pallet::compact] frequency: Frequency,
			#[pallet::compact] name_s: Vec<u8>,
			#[pallet::compact] freezer: <T as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo { 
			let seller = ensure_signed(origin)?;
			let freezer = T::Lookup::lookup(freezer);
			match !Self::verify_new_plan(&id) { 
				true => {
					return Err(Error::<T>::PaymentPlanDoesNotExist); 
				},
				_ => {
					//	Access Payment Plan Details 
					let payment_info = PaymentInfo::<T>::get(id).ok_or(Error::<T>::Unknown)?;
					ensure!(payment_info.merchant == seller, Error::<T>::UnAuthorisedCall);

					let new_name: BoundedVec<u8, T::StringLimit> =
						name_s.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

					PaymentInfo::<T>::insert(
						id,
						PaymentPlan { 
							merchant: payment_info.merchant,
							name: new_name,
							payment_id: payment_info.payment_id,
							required_payment: new_payment,
							total_deposits: payment_info.total_deposits,
							frequency,
							num_subscribers: payment_info.num_subscribers,
							freezer,
							is_frozen: payment_info.is_frozen,
						}
					);
					Self::deposit_event(Event::EditedPaymentPlan { 
						merchant: seller, 
						id, 
						now: T::Moment::now()
					});
				}
			}
			Ok(().into())
		}
		//	If merchant is present, or not, refund the remaining balances to the merchant owner found 
		//	inside the struct 
		#[pallet::weight(10_000)]
		pub fn force_delete_plan(
			origin: OriginFor<T>,
			#[pallet::compact] payment_id: PaymentIndex
		) -> DispatchResult { 
			let maybe_owner: Option<T::AccountId> = match T::ForceOrigin::try_origin(origin) { 
				Ok(_) => None, 
				Err(origin) => Some(ensure_signed(origin)?),
			};
			match !Self::verify_new_plan(&payment_id) { 
				true => {
					return Err(Error::<T>::PaymentPlanDoesNotExist); 
				},
				_ => {
					let payment_info = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;
					//	Check if the Some(Merchant) is the Owner
					if let Some(check_owner) = maybe_owner { 
						ensure!(payment_info.merchant == check_owner, Error::<T>::UnAuthorisedCall);
					}
					//	If total deposit is not zero, return to user 
					if !payment_info.total_deposits == Zero::zero() {
						//	Ensure we are not charging fees when the user decides to collect their payments
						let _ = T::Currency::resolve_creating( 
							&payment_info.merchant, T::Currency::withdraw(
								&Self::fund_account_id(payment_id),
								payment_info.total_deposits,
								WithdrawReasons::TRANSFER,
								ExistenceRequirement::AllowDeath,
							)?
						);
						Self::deposit_event(Event::PaymentRefundedToMerchant { 
							merchant: payment_info.merchant,
							id: payment_id, 
							now: T::Moment::now(),
						});
					}
					//	Delete from storage 
					PaymentInfo::<T>::remove(&payment_id);
					pallet_scheduler::Pallet::<T>::cancel_named(payment_info.name);
					
					Self::deposit_event(Event::PaymentPlanKilled { 
						merchant: maybe_owner,
						id: payment_id, 
						now: T::Moment::now()
					});	
				},
			} 
			Ok(())
		}
		///	Transfer ownership of a payment plan to an approved delegate 
		///	Origin 
		#[pallet::weight(10_000)]
		pub fn transfer_ownership_plan(
			origin: OriginFor<T>,
			#[pallet::compact] payment_id: PaymentIndex,
			delegate: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo { 
			let maybe_owner: Option<T::AccountId> = match T::ForceOrigin::ensure_signed(origin) { 
				Ok(_) => None, 
				Err(origin) => Some(ensure_signed(origin)?)
			};
			let delegate = T::Lookup::lookup(delegate)?;

			match !Self::verify_new_plan(&payment_id) { 
				true => {
					return Err(Error::<T>::PaymentPlanDoesNotExist)
				},	
				_ => {
					let mut payment_info = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;

					if let Some(check_owner) = maybe_owner { 
						ensure!(payment_info.merchant == check_owner, Error::<T>::UnAuthorisedCall);
					}
					payment_info.merchant = Some(delegate); 
					PaymentInfo::<T>::insert(&payment_id, &payment_info);
					
					Self::deposit_event(Event::TransferApproved { 
						owner: maybe_owner, 
						new_owner: delegate,
						id: payment_id,
						now: T::Moment::now()
					});
				}
			}
			Ok(().into())
		}
		///	Issue a new class of payment plans from a priviledged origin 
		/// There will be no assets stored inside of this
		///	Required Payment Will Be Set to Zero 
		#[pallet::weight(10_000)]
		pub fn force_create_new(
			origin: OriginFor<T>,
			owner: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] name: Vec<u8>,
			#[pallet::compact] frequency: Frequency,
			#[pallet::compact] required_payment: T::Balance,
			#[pallet::compact] freezer: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] set_frozen: bool,

		) -> DispatchResult { 
			T::ForceOrigin::ensure_signed(origin)?;	
			let owner = T::Lookup::lookup(origin)?;
			let freezer = T::Lookup::lookup(origin)?;
			let payment_id = Self::next_payment_id();

			match !Self::verify_new_plan(&payment_id) { 
				_ => {
					return Err(Error::<T>::PaymentPlanDoesNotExist);
				},
				true => {
					let bounded_name: BoundedVec<u8, T::StringLimit> =
					name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
					
					let deposit = T::SubmissionDeposit::get();
					
					let new_payment = Self::new_payment_plan()
						.identified_by(payment_id.clone())
						.owned_by(owner.clone())
						.with_name(bounded_name.clone())
						.min_payment()
						.new_deposit(required_payment)
						.payment_frequency(frequency)
						.total_subscribers(Zero::zero())
						.freezer(freezer)
						.freeze_payments(set_frozen)
						.build();

					let imbalance = T::Currency::withdraw(
						&owner, 
						deposit, 
						WithdrawReasons::TRANSFER,
						ExistenceRequirement::AllowDeath
					)?;
					//	Create a fund, imabalance is empty 
					T::Currency::resolve_creating(
						&Self::fund_account_id(payment_id),
						imbalance
					);
					PaymentInfo::<T>::insert(payment_id, new_payment);
					//	Insert to storage map 
					Self::deposit_event(Event::PaymentPlanCreated { 
						merchant: owner,
						id: payment_id, 
						now: T::Moment::now(),
					});
				}
			}

			Ok(())
		}
		///	Block further unpriviledge transfers from an account 
		/// Origin must be Signed and the sender should be the Freezer of the Payment PLan 
		#[pallet::weight(10_000)]
		pub fn freeze_payment_plan(
			origin: OriginFor<T>,
			#[pallet::compact] payment_id: PaymentIndex,
		) -> DispatchResultWithPostInfo {
			let merchant = ensure_signed(origin)?;

			match !Self::verify_new_plan(&payment_id) { 
				true => {
					return Err(Error::<T>::PaymentPlanDoesNotExist);
				},
				_ => { 
					let mut payment_info = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;
					ensure!(payment_info.freezer == merchant, Error::<T>::UnAuthorisedCall);
					payment_info.is_frozen = true;
					
					PaymentInfo::<T>::insert(&payment_id, payment_info);
					Self::deposit_event(Event::Frozen { 
						freezer: merchant,
						id: payment_id, 
						now: T::Moment::now()
					});
				}
			}

			Ok(().into())
		}
		#[pallet::weight(10_000)]
		pub fn unfreeze_payment_plan(
			origin: OriginFor<T>,
			#[pallet::compact] payment_id: PaymentIndex
		) -> DispatchResultWithPostInfo { 
			let maybe_owner = match T::ForceOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)? ),
			};
			match !Self::verify_new_plan(&payment_id) { 
				true => {
					return Err(Error::<T>::PaymentPlanDoesNotExist)
				},	
				_ => {
					let mut payment_info = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;
					ensure!(!payment_info.is_frozen, Error::<T>::AlreadyUnFrozen);

					if let Some(check_owner) = maybe_owner { 
						ensure!(payment_info.freezer == check_owner, Error::<T>::UnAuthorisedCall);
					}
					payment_info.is_frozen = true;
					PaymentInfo::<T>::insert(&payment_id, payment_info);

					Self::deposit_event(Event::UnFrozen { 
						who: maybe_owner,
						id: payment_id, 
						now: T::Moment::now()
					});
				}
			}

			Ok(().into())
		}
		///	Remove User from the Subscription List 
		/// Refund user for any unspent subscription period 
		/// Remove proxy from user to prevent scheduled dispatchables 
		/// Destroy User Subscription 
		#[pallet::weight(10_000)]
		pub fn force_cancel_subscription(
			origin: OriginFor<T>,
			#[pallet::compact] payment_id: PaymentIndex,
			subscriber: <T as StaticLookup>::Source
		) -> DispatchResult { 
			T::ForceOrigin::ensure_signed(origin)?;
			let subscriber = T::Lookup::lookup(subscriber)?;

			match !Self::verify_new_plan(&payment_id) {
				true => {
					return Err(Error::<T>::PaymentPlanDoesNotExist)
				},
				_ => {
					let subscription_info = Subscriptions::<T>::get(&subscriber).ok_or(Error::<T>::Unknown)?;
					let mut info = subscription_info.1;
					ensure!(!info.subscribed_to.is_empty(), Error::<T>::UserNotSubscribed);
					ensure!(!info.subscribed_to.contains(&payment_id), Error::<T>::UserNotSubscribed);
					
					//	Calculate the amount of refund given to user for unspent period 
					if info.required_payment != Zero::zero()  { 
						//	Get the users payment for this CURRENT PERIOD, not the entire paymentsm, ensure merchants are collecting 
						//	User payments on a periodic schedule
						let balance = Self::get_payment_info(payment_id, &subscriber);						
					 
						let begin_period = info.start;
						let payment_due = info.frequency_of.frequency();
						let difference =  payment_due - I32F32::from_num(info.start);
						let ratio = I32F32::from_num(difference)/payment_due;
						
						//	How much the user get to keep 
						let new_ratio = I32F32::from_num(1) - ratio;
						
						//	If the ratio is not 1, then we can refund all unused period 
						if new_ratio != I32F32::from_num(1) { 
							//	Refund on current subscription period, ensure the collection has their scheduled date dispatchable function 
							//	The line below is: 
							//	Refunded amount = CurrentPeriodBalance(1 - Ratio between the starting date and due payment date)
							//	ie RefundedToUser = 100(1 - 0.5 <-- or 50% before finishing the period) => refund 50 to user
							let user_refund = balance.saturating_mul(new_ratio);
							
							// Return funds to caller without charging a transfer fee
							let _ = T::Currency::resolve_into_existing(&subscriber, 
								T::Currency::withdraw(
								&Self::fund_account_id(payment_id), 
									user_refund, 
									WithdrawReasons::TRANSFER, 
									ExistenceRequirement::AllowDeath
								)?
							);
							Self::deposit_event(Event::PaymentRefundedToUser { 
								user: subscriber,
								id: payment_id, 
								now: T::Moment::now()
							});
						}
					} 
					pallet_proxy::Pallet::<T>::remove_proxies(&subscriber);
					//	Delete subscription info 
					Subscriptions::<T>::remove(&subscriber);
					Self::deposit_event(Event::RecurringPaymentCancelled { 
						user: subscriber,
						id: payment_id,
						now: T::Moment::now(),
					});
				}
			}
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn force_payment(
			origin: OriginFor<T>,
			subscriber: <T as StaticLookup>::Source,
			#[pallet::compact] payment_id: PaymentIndex,
		) -> DispatchResultWithPostInfo { 
			let admin = T::ForceOrigin::ensure_signed(origin)?;
			let subscriber = T::Lookup::lookup(subscriber)?;

			match !Self::verify_new_plan(&payment_id) { 
				true => {
					return Err(Error::<T>::PaymentPlanDoesNotExist);
				},	
				_ => {
					let mut payment_info = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;
					let mut subscription_info = Subscriptions::<T>::get(&subscriber).ok_or(Error::<T>::UserDoesNotExist)?;
					let now = frame_system::Pallet::block_number();
					
					//	Check if the user is associated with the subscription info 
					ensure!(subscription_info.1.owner == subscriber, Error::<T>::NotASubscriber);
					//	Check how many users are subscribed to this payment plan, if zero, emit error
					ensure!(payment_info.num_subscribers != Zero::zero(), Error::<T>::NoSubscribersFound);
					//	Check if the payment plan is blocking any transfers
					ensure!(!payment_info.is_frozen, Error::<T>::Frozen);
					//	Check if the user is subscribed to the payment plan 
					ensure!(subscription_info.1.subscribed_to.contains(&payment_id), Error::<T>::UserNotSubscribed);
					//	Check if the user is meant to pay now 
					ensure!(subscription_info.1.next_payment == now, Error::<T>::NotDueYet);
					//	Get users that are subscribed into this payment plan 
					
					let force_call = pallet_proxy::Pallet::proxy(admin, subscriber, Default::default(), 
					//	Force call the user to transfer funds into the child trie
						T::Currency::transfer(
							&subscriber, 
							&Self::fund_account_id(payment_id),
							subscription_info.required_payment,
							ExistenceRequirement::AllowDeath
						)?
					);
					//	If force payment call fails, force_cancel user subscription 
					if force_call.is_err() { 
						//	Revoke user Priviledges
						Self::force_cancel_subscription(
							admin, 					
							payment_id,
							subscriber
						)?;
					}
				},
			}

			Ok(().into())
		}
	} 
	impl<T: Config> Pallet<T> { 
		//	Does the id already exist in our storage? 
		fn verify_new_plan(id: &PaymentIndex) -> bool { 
			PaymentInfo::<T>::contains_key(id)
		}
		//	Create a new payment plan using PaymentPlanBuilder
		fn new_payment_plan() -> PaymentPlanBuilder<T::AccountId, T::Balance> { 
			PaymentPlanBuilder::<T::AccountId, T::Balance>::default()
		}
		//	Create subscription to a Payment Plan 
		fn new_subscription() -> SubscriptionBuilder<T::AccountId, T::Moment, T::Balance> { 
			SubscriptionBuilder::<T::AccountId, T::Moment, T::Balance>::default()
		}
		//	This is where recurring payments are paid into 
		fn fund_account_id(idx: PaymentIndex) -> T::AccountId { 
			T::PalletId::get().into_sub_account(idx)
		}
		//	Track Payment Index
		fn next_payment_id() -> Result<u32, DispatchError> {
			PaymentId::<T>::try_mutate(|index| -> Result<u32, DispatchError> {
				let current_id = *index;
				*index = index.checked_add(1).ok_or(ArithmeticError::Overflow)?;
				Ok(current_id)
			})
		}
		fn next_subscriber_id() -> Result<u32, DispatchError> { 
			SubscriptionId::<T>::try_mutate(|id| -> Result<u32, DispatchError> { 
				let curr = *id;
				*id = id.checked_add(1).ok_or(ArithmeticError::Overflow)?;
				Ok(curr)
			})
		}
		// 	Function to find the id associated with the fund id (child trie)
		//	Each fund stores information about it ***contributors and their ***contributions in a child trie 
		
		//	This helper function calculates the id of the associate child trie 
		fn id_from_index(
			index: PaymentIndex
		) -> child::ChildInfo { 
			let mut buf = Vec::new();
			buf.extend_from_slice(b"payment");
			buf.extend_from_slice(&index.to_le_bytes()[..]);

			child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
		}
		//	Put Payment under a key: user account 
		fn insert_payment(
			index: PaymentIndex, 
			who: &T::AccountId, 
			balance: &T::Balance
		) {
			let id = Self::id_from_index(index);
			who.using_encoded(|b| child::put(&id, b, &balance));
		}
		//	Get the value paid by the user 
		fn get_payment_info(
			index: PaymentIndex, 
			who: &T::AccountId
		) -> BalanceOf<T> {
			let id = Self::id_from_index(index);
			who.using_encoded(|b| child::get_or_default::<BalanceOf<T>>(&id, b))
		}
		fn kill_paymentsystem(index: PaymentIndex) {
			let id = Self::id_from_index(index);
			// The None here means we aren't setting a limit to how many keys to delete.
			// Limiting can be useful, but is beyond the scope of this recipe. For more info, see
			// https://crates.parity.io/frame_support/storage/child/fn.kill_storage.html
			child::kill_storage(&id, None);
		}
	}
	impl<T: Config> PaymentInfo<T> { 

	}
	impl<T: Config> Subscriptions<T> { 
		
	}
}