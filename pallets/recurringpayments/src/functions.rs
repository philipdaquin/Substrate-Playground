// This file is part of Gamme Finance.

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

//! Functions for the RecurringPayment pallet.
use frame_support::pallet_prelude::*;
use frame_support::traits::schedule::DispatchTime;
use frame_system::pallet_prelude::*;
use frame_support::storage::ChildTriePrefixIterator;
use sp_runtime::traits::StaticLookup;
use frame_support::{ensure, traits::Get};
use frame_support::{ BoundedVec};

use super::*;
// use pallet::{Store, PaymentInfo, SubscriptionId, SubscriptionIndex, Subscriptions, Pallet, };
// use crate::types::*;
// use crate::builder::*;

// impl<T: Config> Pallet<T> { 
		
//     //	Does the id already exist in our storage? 
//     // pub(crate) fn verify_new_plan(id: &PaymentIndex) -> bool { 
//     //     PaymentInfo::<T>::contains_key(id)
//     // }

//     //	Create a new payment plan using PaymentPlanBuilder
//     // pub(super) fn new_payment_plan() -> PaymentPlanBuilder<T::AccountId, T::Balance> { 
//     //     PaymentPlanBuilder::<T::AccountId, T::Balance>::default()
//     // }

//     //	Create subscription to a Payment Plan 
//     // pub(super) fn new_subscription() -> SubscriptionBuilder<T::AccountId, T::Moment, T::Balance> { 
//     //     SubscriptionBuilder::<T::AccountId, T::Moment, T::Balance>::default()
//     // }

//     //	This is where recurring payments are paid into 
//     pub(super) fn fund_account_id(idx: PaymentIndex) -> T::AccountId { 
//         T::PalletId::get().into_sub_account(idx)
//     }
//     //	Track Payment Index
//     pub(super) fn next_payment_id() -> Result<u32, DispatchError> {
//         PaymentId::<T>::try_mutate(|index| -> Result<u32, DispatchError> {
//             let current_id = *index;
//             *index = index.checked_add(1).ok_or(ArithmeticError::Overflow)?;
//             Ok(current_id)
//         })
//     }
//     pub(super) fn next_subscriber_id() -> Result<u32, DispatchError> { 
//         SubscriptionId::<T>::try_mutate(|id| -> Result<u32, DispatchError> { 
//             let curr = *id;
//             *id = id.checked_add(1).ok_or(ArithmeticError::Overflow)?;
//             Ok(curr)
//         })
//     }
//     pub(super) fn trie_iterator(id: &PaymentIndex) -> ChildTriePrefixIterator<(T::AccountId, (BalanceOf<T>, Vec<u8>))> { 
//         ChildTriePrefixIterator::<_>::with_prefix_over_key::<Identity>(
//             &Self::id_from_index(id),
//             &[],
//         )
//     }
//     // 	Function to find the id associated with the fund id (child trie)
//     //	Each fund stores information about it ***contributors and their ***contributions in a child trie 
    
//     //	This helper function calculates the id of the associate child trie 
//     pub(super) fn id_from_index(
//         index: &PaymentIndex
//     ) -> child::ChildInfo { 
//         let mut buf = Vec::new();
//         buf.extend_from_slice(b"payment");
//         buf.extend_from_slice(&index.to_le_bytes()[..]);

//         child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
//     }
//     //	Put Payment under a key: user account 
//     pub(super) fn insert_payment(
//         index: PaymentIndex, 
//         who: &T::AccountId, 
//         balance: &BalanceOf<T>
//     ) {
//         let id = Self::id_from_index(&index);
//         who.using_encoded(|b| child::put(&id, b, &balance));
//     }
//     //	Get the value paid by the user 
//     pub(super) fn get_payment_info(
//         index: PaymentIndex, 
//         who: &T::AccountId
//     ) -> BalanceOf<T> {
//         let id = Self::id_from_index(&index);
//         who.using_encoded(|b| child::get_or_default::<BalanceOf<T>>(&id, b))
//     }
//     pub(super) fn kill_paymentsystem(index: PaymentIndex) {
//         let id = Self::id_from_index(&index);
//         // The None here means we aren't setting a limit to how many keys to delete.
//         // Limiting can be useful, but is beyond the scope of this recipe. For more info, see
//         // https://crates.parity.io/frame_support/storage/child/fn.kill_storage.html
//         child::kill_storage(&id, None);
//     }
//     pub(super) fn calculate_next_duedate(
//         start: T::BlockNumber, 
//         frequency: Frequency,
//     ) -> Result<T::BlockNumber, DispatchError> {
//         let now = frame_system::Pallet::<T>::block_number();
//         let freq: u32 = frequency.frequency();
//         let mut due_date = start.saturating_add(T::BlockNumber::from(freq));
        
//         if due_date == now { 
//             due_date = now.saturating_add(T::BlockNumber::from(freq))
//         }
//         //  Return the due_date for BlockNumber 
//         Ok(due_date)
//     }
//     //	Calculate the portion of funds that have exhausted 
//     pub(super) fn calculate_leftover(

//     ) -> Result<(), DispatchError>{

//         Ok(())
//     }

//     pub(super) fn do_create(
//         payment_id: PaymentIndex,
//         name: Vec<u8>,
//         required_payment: BalanceOf<T>,
//         frequency: Frequency,
//         merchant: AccountId<T>,
//         freezer: Option<<T::Lookup as StaticLookup>::Source>,
//         schedule_periodic_collection: Option<Frequency>,
//     //	collection_portion: Option<Portion>,
//         deposit: BalanceOf<T>,
//         event: Event<T>,
//     ) -> DispatchResult {
                
//         ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanAlreadyExist);
//         let bounded_name: BoundedVec<u8, T::StringLimit> =
//             name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
//         let freezer = freezer.map(T::Lookup::lookup).transpose()?;
        
//         PaymentInfo::<T>::insert(
//             payment_id,
//             PaymentPlan::<AccountId<T>, BalanceOf<T>, BoundedVec<u8, T::StringLimit>> { 
//                     merchant: merchant.clone(),
//                     name: bounded_name.clone(),
//                     payment_id,
//                     required_payment,
//                     total_deposits: Zero::zero(),
//                     frequency,
//                     num_subscribers: Zero::zero(),
//                     freezer,
//                     is_frozen: false,
//                     schedule_periodic_collection: schedule_periodic_collection.clone()
//             }
//         );
        
//         let imbalance = <T as pallet::Config>::Currency::withdraw(
//             &merchant, 
//             deposit, 
//             WithdrawReasons::TRANSFER,
//             ExistenceRequirement::AllowDeath
//         )?;
//         //	Create a fund, imbalance is empty 
//         <T as pallet::Config>::Currency::resolve_creating(
//             &Self::fund_account_id(payment_id),
//             imbalance
//         );
//         let now = frame_system::Pallet::<T>::block_number();
        
//         //  Schedule Periodic Collections
//         //	TODO: 'Change maybe_period'
//         T::Scheduler::schedule_named(
//             bounded_name.to_vec(),
//             DispatchTime::At(now),
//             None,
//             63,
//             frame_system::RawOrigin::Root.into(),
//             Call::collect_payments {
//                 payment_id, 
//                 schedule_periodic_collection: schedule_periodic_collection.clone(), 
//         //		specified_portion: collection_portion,
//             }.into()
//         );

//         Self::deposit_event(event);
//         Ok(())

//     }
//     pub(super) fn do_collect_payments(
//         merchant: AccountId<T>, 
//         payment_id: PaymentIndex,
//     //	specified_portion: Option<Portion>,
//         event: Event<T>,
//     ) -> DispatchResult {
//         ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
        
//         let mut payment_info = PaymentInfo::<T>::get(&payment_id);
        
//         ensure!(merchant == payment_info.merchant, Error::<T>::UnAuthorisedCall);
//         ensure!(!payment_info.total_deposits.is_zero(), Error::<T>::InsufficientBalance);

//         // let total_balance = payment_info.total_deposits;
//         // //  only collect 50% * total deposit = update the storage 
//         // if let Some(amount) = specified_portion { 
//         // 	let mut total = payment_info.total_deposits;
//         // 	let per_deposit = amount.portion().saturating_mul(total); 

//         // 	let mut user_requested_amount: BalanceOf<T> = total.saturating_mul(per_deposit.into());
//         // 	let mut new_total_deposit_in_storage = total.checked_sub(&user_requested_amount).ok_or(ArithmeticError::Underflow)?;

//         // 	payment_info.total_deposits = new_total_deposit_in_storage;

//         // 	PaymentInfo::<T>::insert(payment_id, payment_info);
//         // 	 //	Ensure we are not chargin fees when the user decides to collect their payments
//         // 	}
//         // }

//         let _ = <T as pallet::Config>::Currency::resolve_creating( 
//             &payment_info.merchant, 
//             <T as pallet::Config>::Currency::withdraw(
//                 &Self::fund_account_id(payment_id),
//                 payment_info.total_deposits,
//                 WithdrawReasons::TRANSFER,
//                 ExistenceRequirement::AllowDeath,
//                 )?
//         );
//         PaymentInfo::<T>::insert(payment_id, payment_info);

//         Self::deposit_event(event);
        
//         Ok(())
//     }
//     pub(super) fn do_transfer_ownership(
//         maybe_owner: Option<AccountId<T>>,
//         payment_id: PaymentIndex, 
//         delegate: AccountId<T>,
//         event: Event<T>, 
//     ) -> DispatchResult {
//         ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
        
//         let mut payment_info = PaymentInfo::<T>::get(&payment_id);

//         if let Some(check_owner) = maybe_owner { 
//             ensure!(payment_info.merchant == check_owner, Error::<T>::UnAuthorisedCall);
//         }
//         payment_info.merchant = delegate; 
//         PaymentInfo::<T>::insert(&payment_id, &payment_info);
        
//         Self::deposit_event(event);

//         Ok(())
//     }
    
//     pub(super) fn do_subscribed(
//         payment_id: PaymentIndex, 
//         min_payment: BalanceOf<T>,
//         num_frequency: Option<u32>,
//         subscriber: AccountId<T>,
//         event_payment_sent: Event<T>,
//         event_sub_created: Event<T>,
//     ) -> DispatchResult {
        
//         ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
        
//         //	Access Payment plan information  
//         let mut payment_plan = PaymentInfo::<T>::get(&payment_id);
//         // ensure the user has enough to supplement for required_payment
//         ensure!(min_payment >= payment_plan.required_payment, Error::<T>::InsufficientBalance); 
//         //	check if the PaymentPlan is Frozen
//         ensure!(!payment_plan.is_frozen, Error::<T>::Frozen);
//         //	else -> Transfer funds into fund_index 
//         <T as pallet::Config>::Currency::transfer(
//             &subscriber, 
//             &Self::fund_account_id(payment_id),
//             min_payment,
//             ExistenceRequirement::AllowDeath
//         )?;
//         payment_plan.total_deposits += min_payment;
//         //	Increment Number of Subscribers 
//         payment_plan.num_subscribers.checked_add(1).ok_or(ArithmeticError::Overflow)?;
        
//         let payment = Self::get_payment_info(payment_id, &subscriber);
        
//         let payment = payment.saturating_add(min_payment);
//         Self::insert_payment(payment_id, &subscriber, &payment);

//         let starting_block = frame_system::Pallet::<T>::block_number();
//         let mut due_date = Self::calculate_next_duedate(starting_block, payment_plan.frequency.clone())?;
//         let mut sub_list: Vec<u32> = vec![];
        
//         sub_list.push(payment_id);
        
//         let subscriber_id = Self::next_subscriber_id()?;
//         //	Update Storage Subscriptions 
//         Subscriptions::<T>::insert(
//             subscriber.clone(),
//             Subscription { 
//                 id: subscriber_id,
//                 owner: subscriber.clone(),
//                 start: starting_block,
//                 required_payment: min_payment,
//                 next_payment: due_date,
//                 frequency_of: payment_plan.frequency.clone(),
//                 num_frequency: None, 
//                 subscribed_to: sub_list.clone() 					
//             }
//         );
//         PaymentInfo::<T>::insert(payment_id, &payment_plan);
//         Self::deposit_event(event_payment_sent);
    
//         //	Force Payments Takes in Abstract Accounts that are subscribed to payments 
//         //	We convert the Subscriber's account back into source and create a create a scheduled dispatchable fucntion  
//         let subscriber =  T::Lookup::unlookup(subscriber);
//         //	Schedule a dispatchable function based on frequency
//         //	Schedule name is based on the PaymentPlan Name
//         //	This is specified according to the users preference 
//         //* Needs to be Reviewed */
//         T::Scheduler::schedule_named(
//             payment_plan.name.to_vec(),
//             DispatchTime::At(due_date),
//             None, 
//             Default::default(),
//             frame_system::RawOrigin::Root.into(),
//             Call::force_payment { 
//                 subscriber,
//                 payment_id,
//                 required_payment: min_payment
//             }.into()
//         );
        
//         Self::deposit_event(event_sub_created);

//         Ok(())
//     }
//     pub(super) fn do_cancel(
//         subscriber: AccountId<T>,
//         payment_id: PaymentIndex,
//         event_refund: Event<T>,
//         event_cancelled: Event<T>,
//     ) -> DispatchResult {
//         ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
//         //	Get the users contribution 
//         let balance = Self::get_payment_info(payment_id, &subscriber);
//         let subscriber_info = Subscriptions::<T>::get(&subscriber);
//         let now = frame_system::Pallet::<T>::block_number();
//         let begin_period = subscriber_info.start;
//         let payment_due = subscriber_info.next_payment;
        
//         let amount_refunded = Self::calculate_ratio_start_end(
//             begin_period,
//             payment_due,
//             &now, 
//             balance
//         );

//         if amount_refunded.is_err() { 
//             let _ = <T as pallet::Config>::Currency::resolve_into_existing(&subscriber, 
//                 <T as pallet::Config>::Currency::withdraw(
//                 &Self::fund_account_id(payment_id), 
//                     amount_refunded.unwrap(), 
//                     WithdrawReasons::TRANSFER, 
//                     ExistenceRequirement::AllowDeath
//                 )?
//             );
            
//             Self::deposit_event(event_refund);
//         }
//         //	if we get a balance, then we refund it back to the user 
        
//         //	Remove Proxy inside the user 
//         pallet_proxy::Pallet::<T>::remove_proxies(frame_system::RawOrigin::Root.into());
//         //	Remove schedule dispatchable
//         let payment_info = PaymentInfo::<T>::get(payment_id);
        
//         pallet_scheduler::Pallet::<T>::cancel_named(frame_system::RawOrigin::Root.into(), 
//             payment_info.name.to_vec());
        
//         Self::deposit_event(event_cancelled);
        
//         Ok(())
//     } 
//     pub(super) fn do_edit_plan(
//         id: PaymentIndex,
//         name_s: Vec<u8>,
//         new_payment: BalanceOf<T>,
//         frequency: Frequency,
//         seller: AccountId<T>,
//         freezer: Option<<T::Lookup as StaticLookup>::Source>,
//         schedule_periodic_collection: Option<Frequency>,
//     //	collection_portion: Option<Portion>,
//         event: Event<T>,
//     ) -> DispatchResult { 
        
//         ensure!(!PaymentInfo::<T>::contains_key(&id), Error::<T>::PaymentPlanDoesNotExist);
//         //	Access Payment Plan Details 
//         let mut payment_info = PaymentInfo::<T>::get(id);
//         ensure!(payment_info.merchant == seller, Error::<T>::UnAuthorisedCall);

//         let new_name: BoundedVec<u8, T::StringLimit> =
//             name_s.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
//         let freezer = freezer.map(T::Lookup::lookup).transpose()?;
        
//         PaymentInfo::<T>::insert(
//             id,
//             PaymentPlan::<AccountId<T>, BalanceOf<T>, BoundedVec<u8, T::StringLimit>> { 
//                 merchant: payment_info.merchant,
//                 name: new_name,
//                 payment_id: payment_info.payment_id,
//                 required_payment: new_payment,
//                 total_deposits: payment_info.total_deposits,
//                 frequency,
//                 num_subscribers: payment_info.num_subscribers,
//                 freezer,
//                 is_frozen: payment_info.is_frozen,
//                 schedule_periodic_collection		
//             }
//         );
//         //	TODO Collection Portion 

//         Self::deposit_event(event);
        
//         Ok(())
//     }
//     pub(super) fn do_delete(
//         maybe_owner: Option<AccountId<T>>,
//         payment_id: PaymentIndex,
//         event_payment_killed: Event<T>
//     ) -> DispatchResult { 
//         ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
//         let now = frame_system::Pallet::<T>::block_number();
//         let payment_info = PaymentInfo::<T>::get(&payment_id);
//         //	Check if the Some(Merchant) is the Owner
//         if let Some(check_owner) = maybe_owner { 
//             ensure!(payment_info.merchant == check_owner, Error::<T>::UnAuthorisedCall);
//         }
//         //	If total deposit is not zero, return to user 
//         if !payment_info.total_deposits.is_zero() {
//             //	Ensure we are not charging fees when the user decides to collect their payments
//             let _ = <T as pallet::Config>::Currency::resolve_creating( 
//                 &payment_info.merchant, <T as pallet::Config>::Currency::withdraw(
//                     &Self::fund_account_id(payment_id),
//                     payment_info.total_deposits,
//                     WithdrawReasons::TRANSFER,
//                     ExistenceRequirement::AllowDeath,
//                 )?
//             );
//             Self::deposit_event(Event::PaymentRefundedToMerchant { 
//                 merchant: payment_info.merchant,
//                 id: payment_id, 
//                 now
//             });
//         }
//         //	Delete from storage 
//         PaymentInfo::<T>::remove(&payment_id);
//         T::Scheduler::cancel_named(payment_info.name.to_vec());

//         Self::deposit_event(event_payment_killed);	

//         Ok(())
//     }
//     pub(super) fn do_freeze(
//         merchant: Option<AccountId<T>>, 
//         payment_id: PaymentIndex,
//         event_freeze: Event<T>,
//     ) -> DispatchResult {
//         ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);

//         let mut payment_info = PaymentInfo::<T>::get(&payment_id);
        
//         if let Some(check_owner) = merchant { 
//             ensure!(payment_info.freezer == Some(check_owner), Error::<T>::UnAuthorisedCall);
//         }
        
//         payment_info.is_frozen = true;
        
//         PaymentInfo::<T>::insert(&payment_id, payment_info);
        
//         Self::deposit_event(event_freeze);        

//         Ok(())
//     }
//     pub(super) fn do_unfreeze(
//         maybe_owner: Option<AccountId<T>>,
//         payment_id: PaymentIndex,
//         event_unfreeze: Event<T>,
//     ) -> DispatchResult { 
//         let mut payment_info = PaymentInfo::<T>::get(&payment_id);
//         ensure!(!payment_info.is_frozen, Error::<T>::AlreadyUnFrozen);

//         if let Some(check_owner) = maybe_owner { 
//             ensure!(payment_info.freezer == Some(check_owner), Error::<T>::UnAuthorisedCall);
//         }
//         payment_info.is_frozen = true;
        
//         PaymentInfo::<T>::insert(&payment_id, payment_info);

//         Self::deposit_event(event_unfreeze);
//         Ok(())
//     } 
//     pub(super) fn do_force_cancel(
//         subscriber: AccountId<T>, 
//         payment_id: PaymentIndex,
//     ) -> DispatchResult {
        
//         let now = frame_system::Pallet::<T>::block_number();
//         Self::do_cancel(
//             subscriber.clone(),
//             payment_id,
//             Event::PaymentRefundedToUser { 
//                 user: subscriber.clone(),
//                 id: payment_id, 
//                 now,
//             },
//             Event::RecurringPaymentCancelled { 
//                 user: subscriber.clone(),
//                 id: payment_id,
//                 now,
//             }
//         )
//     }
//     pub(super) fn do_force_pay(
//         subscriber: AccountId<T>,
//         payment_id: PaymentIndex,
//         required_payment: BalanceOf<T>,
//         event_cancelled: Event<T>,
//         event_paid: Event<T>,
//     ) -> DispatchResult {		
//         ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
        
//         //let admin = T::ForceOrigin::ensure_origin(admin)?;
//         let subscriber = subscriber; 
//         let mut payment_info = PaymentInfo::<T>::get(&payment_id);
//         let mut subscription_info = Subscriptions::<T>::get(subscriber.clone());
//         let now = frame_system::Pallet::<T>::block_number();

//         //  Check if the required payments matches with the users subcription 
//         ensure!(subscription_info.required_payment == payment_info.required_payment, Error::<T>::InvalidPayment);
//         //	Check if the user is associated with the subscription info 
//         ensure!(subscription_info.owner == subscriber.clone(), Error::<T>::NotASubscriber);
//         //	Check how many users are subscribed to this payment plan, if zero, emit error
//         ensure!(!payment_info.num_subscribers.is_zero(), Error::<T>::NoSubscribersFound);
//         //	Check if the payment plan is blocking any transfers
//         ensure!(!payment_info.is_frozen, Error::<T>::Frozen);
//         //	Check if the user is subscribed to the payment plan 
//         ensure!(subscription_info.subscribed_to.contains(&payment_id), Error::<T>::UserNotSubscribed);
//         //	Check if the user is meant to pay now 
//         ensure!(subscription_info.next_payment == now, Error::<T>::NotDueYet);
//         //	Get users that are subscribed into this payment plan 
        
//         //	Force call the user to transfer funds into the child trie// Root Call
//         //	This Assumes the Subscriber has delegated Proxy to the Root 
//         //	If the call fails, we remove subscription info, proxy calls, scheduled dispatchables and memberships 
//         if Self::transfer_payment(
//             frame_system::RawOrigin::Root.into(),  T::Lookup::unlookup(subscriber.clone()), payment_id,required_payment,
//         ).is_err() { 
//             Subscriptions::<T>::remove(subscriber.clone());
//             //	Remove Proxy inside the user 
//             pallet_proxy::Pallet::<T>::remove_proxies(frame_system::RawOrigin::Root.into());
//             //	Remove schedule dispatchable
//             T::Scheduler::cancel_named(payment_info.name.to_vec()).map_err(|_| Error::<T>::PaymentPlanDoesNotExist)?;
//             //  Cancel User Membership
//             Self::deposit_event(event_cancelled);
            
//         } else { 
//             Self::deposit_event(event_paid)

//             //  Reschedule next the payment 
//         }
//         Ok(())
//     }
//     pub(super) fn force_default(
//         maybe_owner: Option<AccountId<T>>,
//         payment_id: PaymentIndex, 
//         name: Vec<u8>,
//         new_owner: <T::Lookup as StaticLookup>::Source,
//         min_balance: BalanceOf<T>,
//         frequency: Frequency,
//         freezer: Option<<T::Lookup as StaticLookup>::Source>,
//         is_frozen: bool,
//         schedule_periodic_collection: Option<Frequency>,
//         event_edit: Event<T>,
//     ) -> DispatchResult {
//         ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
//         let mut payment_info = PaymentInfo::<T>::get(&payment_id);
        
//         let bounded_name: BoundedVec<u8, T::StringLimit> =
//             name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
//         let freezer = freezer.map(T::Lookup::lookup).transpose()?;

//         PaymentInfo::<T>::try_mutate_exists(payment_id, |info| { 
//             let mut curr = info.as_mut().ok_or(Error::<T>::Unknown)?;
//             curr.merchant = T::Lookup::lookup(new_owner)?;
//             curr.name = bounded_name;
//             curr.required_payment = min_balance;
//             curr.frequency = frequency;
//             curr.num_subscribers = 0;
//             curr.freezer = freezer;
//             curr.schedule_periodic_collection = schedule_periodic_collection;
//             curr.is_frozen = is_frozen;
            
//             *info = Some(curr.clone()); 

//             Self::deposit_event(event_edit);
//             Ok(())
//         })
//     }
//     pub(super) fn refund_to_users(
//         maybe_owner: Option<AccountId<T>>,
//         payment_id: PaymentIndex,
//     ) -> DispatchResult { 

//         ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
//         let mut refunded = 0;
//         let mut payment_info = PaymentInfo::<T>::get(payment_id.clone());
//         if let Some(check_owner) = maybe_owner.clone() { 
//             ensure!(payment_info.merchant == check_owner, Error::<T>::Unknown);
//         }
//         // Return all user funds 
//         let subscriptions = Self::trie_iterator(&payment_id);
//         for (user, (balance, _)) in subscriptions { 
//             <T as pallet::Config>::Currency::transfer(&Self::fund_account_id(payment_id),
//                 &user, 
//                 balance, 
//                 ExistenceRequirement::AllowDeath,
//             )?;
//             payment_info.total_deposits = payment_info.total_deposits.saturating_sub(balance);
//             refunded += 1;
//         }
//         PaymentInfo::<T>::insert(&payment_id, payment_info.clone());
//         let now = frame_system::Pallet::<T>::block_number();	
//         //let sender = maybe_owner.unwrap_or();
//         if refunded == payment_info.num_subscribers { 
//             Self::deposit_event(Event::RefundedUsers { 
//                 sender: maybe_owner.unwrap().clone(),
//                 id: payment_id, 
//                 now,
//             })
//         } else { 
//             Self::deposit_event(Event::<T>::PartiallyRefunded { 
//                 sender: maybe_owner.unwrap(),
//                 id: payment_id, 
//                 now,
//             })
//         }
//         Ok(())
//     }	
//     //	Generic Transfer Function 	
//     pub(super) fn transfer_payment(
//         origin: OriginFor<T>,
//         subscriber: <T::Lookup as StaticLookup>::Source,
//         payment_id: PaymentIndex,
//         min_payment: BalanceOf<T>,
//     ) -> DispatchResult { 
//         ensure_root(origin)?;
//         let subscriber = T::Lookup::lookup(subscriber)?;

//         ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
//         let mut payment_info = PaymentInfo::<T>::get(payment_id);
//         // Transfer 
//         <T as pallet::Config>::Currency::transfer(
//             &subscriber,
//             &Self::fund_account_id(payment_id),
//             min_payment,
//             ExistenceRequirement::AllowDeath
//         )?;
//         //	Update Storage and Trie
//         payment_info.total_deposits += min_payment;
//         PaymentInfo::<T>::insert(payment_id, payment_info);
//         let new_balance = Self::get_payment_info(payment_id, &subscriber);
//         new_balance.saturating_add(min_payment);
//         Self::insert_payment(payment_id, &subscriber, &min_payment);
//         Ok(())
//     }
//     //	Calculate the amount to be refunded if the the user wishes to leave early  
//     pub(super) fn calculate_ratio_start_end(
//         start: T::BlockNumber,
//         next_payment: T::BlockNumber,
//         now: T::BlockNumber,
//         curr_balance: BalanceOf<T>,
//     ) -> Result<BalanceOf<T>, DispatchError> { 
//         ensure!(next_payment != *now, Error::<T>::MinCannotBeZero);
//         //	Calculate the percent from start/ end 
//         //	let ratio = Permill::from_rational(subscription_info.next_payment, subscription_info.start); 
//         let elapsed_blocknumber = next_payment - start;
//         let elapsed_time_u32: u32 = TryInto::try_into(elapsed_blocknumber).ok().expect("Failed To Convert");

//         let remaining_ratio: u32 = 1u32 - elapsed_time_u32;

//         Ok(curr_balance * remaining_ratio.into())

//     }
//     fn percentage() -> I32F32 { 
//         I32F32::from_num(1)
//     }
// }