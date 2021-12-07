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

use frame_support::storage::ChildTriePrefixIterator;
use sp_runtime::traits::StaticLookup;

use super::*;

use crate::types::*;
use crate::builder::*;

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
    fn trie_iterator(id: &PaymentIndex) -> ChildTriePrefixIterator<(T::AccountId, (BalanceOf<T>, Vec<u8>))> { 
        ChildTriePrefixIterator::<_>::with_prefix_over_key::<Identity>(
			&Self::id_from_index(id),
			&[],
		)
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
    pub(super) fn do_create(
        payment_id: PaymentIndex,
        name: Vec<u8>,
        required_payment: T::Balance,
        frequency: Frequency,
        merchant: T::AccountId,
        freezer: Option<T::AccountId>,
        schedule_periodic_collection: Frequency,
        collection_portion: Option<Portion>,
        deposit: BalanceOf<T>,
        event: Event<T>,
    ) -> DispatchResult {
                
        ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanAlreadyExist);
        let bounded_name: BoundedVec<u8, T::StringLimit> =
			name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
        
        PaymentInfo::<T>::insert(
            payment_id,
            PaymentPlan { 
                    merchant,
                    name: bounded_name.clone(),
                    payment_id,
                    required_payment,
                    total_deposits: Zero::zero(),
                    frequency,
                    num_subscribers: Zero::zero(),
                    freezer: freezer.unwrap_or(merchant),
                    is_frozen: false,
                    schedule_periodic_collection
            }
        );
        let imbalance = T::Currency::withdraw(
            &merchant, 
            deposit, 
            WithdrawReasons::TRANSFER,
            ExistenceRequirement::AllowDeath
        )?;
        //	Create a fund, imbalance is empty 
        T::Currency::resolve_creating(
            &Self::fund_account_id(payment_id),
            imbalance
        );
        let now = frame_system::Pallet::<T>::block_number();
        //  Schedule Periodic Collections
        pallet_scheduler::Pallet::<T>::schedule_named(
            merchant,
            name,
            now,
            Some(schedule_periodic_collection.frequency, Default::default()),
            Default::Default(),
            Self::do_collect_payments(
                merchant,
                payment_id,
                collection_portion,
                event
            )
        );

        Self::deposit_event(event);
        Ok(())

    }
    pub(super) fn do_collect_payments(
        merchant: T::AccountId, 
        payment_id: PaymentIndex,
        specified_portion: Option<Portion>,
        event: Event<T>,
    ) -> DispatchResult {
        ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
        
        let mut payment_info = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;
		
        ensure!(merchant == payment_info.merchant, Error::<T>::UnAuthorisedCall);
        ensure!(!payment_info.total_deposits == Zero::zero(), Error::<T>::InsufficientBalance);

        let total_balance = payment_info.total_deposits;
        //  only collect 50% * total deposit = update the storage 
        if let Some(amount) = specified_portion { 
            let per_deposit = amount.portion();
            let mut total = payment_info.total_deposits;
            let mut user_requested_amount: T::Balance = total.saturating_mul(per_deposit);
            let mut new_total_deposit_in_storage = total.checked_sub(user_requested_amount).ok_or(ArithmeticError::Underflow)?;

            payment_info.total_deposits = new_total_deposit_in_storage;

            PaymentInfo::<T>::insert(payment_id, payment_info);
             //	Ensure we are not chargin fees when the user decides to collect their payments
		    let _ = T::Currency::resolve_creating( 
			&payment_info.merchant, 
            T::Currency::withdraw(
				&Self::fund_account_id(payment_id),
				user_requested_amount,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
		    	)?
		    );
        }
        
		Self::deposit_event(event);
		
        Ok(())
    }
    pub(super) fn do_transfer_ownership(
        maybe_owner: Option<T::AccountId>,
        payment_id: PaymentIndex, 
        delegate: T::AccountId,
        event: Event<T>, 
    ) -> DispatchResult {
        ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
        
        let mut payment_info = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;

        if let Some(check_owner) = maybe_owner { 
            ensure!(payment_info.merchant == check_owner, Error::<T>::UnAuthorisedCall);
        }
        payment_info.merchant = Some(delegate); 
        PaymentInfo::<T>::insert(&payment_id, &payment_info);
        
        Self::deposit_event(event);

        Ok(())
    }
    
    pub(super) fn do_subscribed(
        payment_id: PaymentIndex, 
        min_payment: T::Balance,
        num_frequency: Option<u32>,
        subscriber: T::AccountId, 
        event_payment_sent: Event<T>,
        event_sub_created: Event<T>,
    ) -> DispatchResult {
        ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);

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

        let starting_block = frame_system::Pallet::<T>::block_number();
        
        let start_date: Moment = T::Moment::now();
		let next_payment = payment_plan.frequency.frequency()
			.checked_add(start_date)
			.ok_or(ArithmeticError::Overflow)?;
		// Store paymentid into subcription list of user 
		let mut sub_list = vec![];
		
		sub_list.push(payment_id);
        
        let subscriber_id = Self::next_subscriber_id();

        Subscriptions::<T>::insert(&subscriber, 
            (&subscriber_id, Subscription { 
                    owner: subscriber,
                    start: start_date,
                    required_payment: min_payment,
                    next_payment,
                    frequency_of: payment_plan.frequency,
                    num_frequency: num_frequency.unwrap_or_default(),
                    subscribed_to: sub_list,
                }));
        PaymentInfo::<T>::insert(payment_id, &payment_plan);
        Self::deposit_event(event_payment_sent);

        //	Add proxy delegate for the user to allow for scheduled dispatchables
        pallet_proxy::Pallet::<T>::add_proxy(
            subscriber, 
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
            Some(payment.frequency, num_frequency.unwrap_or_default()),
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
        Self::deposit_event(event_sub_created);

        Ok(())
    }
    pub(super) fn do_cancel(
        subscriber: T::AccountId,
        payment_id: PaymentIndex,
        event_refund: Event<T>,
        event_cancelled: Event<T>,
    ) -> DispatchResult {
        ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);

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
			Self::deposit_event(event_refund);
		}
		//	Remove Proxy inside the user 
		pallet_proxy::Pallet::<T>::remove_proxies(&subscriber);
		//	Remove schedule dispatchable
		let payment_info = PaymentInfo::<T>::get(payment_id);
		
        pallet_scheduler::Pallet::<T>::cancel_named(subscriber, payment_info.name);
        
        Self::deposit_event(event_cancelled);
        
        Ok(())
    } 
    pub(super) fn do_edit_plan(
        id: PaymentIndex,
        name_s: Vec<u8>,
        new_payment: T::Balance,
        frequency: Frequency,
        seller: T::AccountId,
        freezer: Option<T::AccountId>,
        schedule_periodic_collection: Frequency,
        collection_portion: Option<Portion>,
        event: Event<T>,
    ) -> DispatchResult { 
        
        ensure!(!PaymentInfo::<T>::contains_key(&id), Error::<T>::PaymentPlanDoesNotExist);
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
                schedule_periodic_collection
            }
        );
        Self::deposit_event(event);
        
        Ok(())
    }
    pub(super) fn do_delete(
        maybe_owner: Option<T::AccountId>,
        payment_id: PaymentIndex,
        event_payment_killed: Event<T>
    ) -> DispatchResult { 
        ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
        
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
        pallet_scheduler::Pallet::<T>::cancel_named(maybe_owner, payment_info.name);
        
        Self::deposit_event(event_payment_killed);	

        Ok(())
    }
    pub(super) fn do_freeze(
        merchant: T::AccountId, 
        payment_id: PaymentIndex,
        event_freeze: Event<T>,
    ) -> DispatchResult {
        ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);

        let mut payment_info = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;
        ensure!(payment_info.freezer == merchant, Error::<T>::UnAuthorisedCall);
        payment_info.is_frozen = true;
        
        PaymentInfo::<T>::insert(&payment_id, payment_info);
        
        Self::deposit_event(event_freeze);        

        Ok(())
    }
    pub(super) fn do_unfreeze(
        maybe_owner: Option<T::AccountId>,
        payment_id: PaymentIndex,
        event_unfreeze: Event<T>,
    ) -> DispatchResult { 
        let mut payment_info = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;
        ensure!(!payment_info.is_frozen, Error::<T>::AlreadyUnFrozen);

        if let Some(check_owner) = maybe_owner { 
            ensure!(payment_info.freezer == check_owner, Error::<T>::UnAuthorisedCall);
        }
        payment_info.is_frozen = true;
        PaymentInfo::<T>::insert(&payment_id, payment_info);

        Self::deposit_event(event_unfreeze);
        Ok(())
    } 
    pub(super) fn do_force_cancel(
        subscriber: T::AccountId, 
        payment_id: PaymentIndex,
    ) -> DispatchResult {
        
        Self::do_cancel(
            subscriber,
            payment_id,
            Event::PaymentRefundedToUser { 
                user: subscriber,
                id: payment_id, 
                now: T::Moment::now()
            },
            Event::RecurringPaymentCancelled { 
                user: subscriber,
                id: payment_id,
                now: T::Moment::now(),
            }
        )
    }
    pub(super) fn do_force_pay(
        subscriber: T::AccountId,
        admin: T::AccountId, 
        payment_id: PaymentIndex,
        event_cancelled: Event<T>,
    ) -> DispatchResult {
        ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
        
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
        
        //	Force call the user to transfer funds into the child trie
        let force_call = T::Currency::force_transfer(
            admin,
            &subscriber, 
            &Self::fund_account_id(payment_id),
            subscription_info.required_payment,)?;
    
        //	If force payment call fails, force_cancel user subscription 
        if force_call.is_err() { 
            Subscriptions::<T>::remove(subscriber);
            //	Remove Proxy inside the user 
            pallet_proxy::Pallet::<T>::remove_proxies(&subscriber);
            //	Remove schedule dispatchable
            pallet_scheduler::Pallet::<T>::cancel_named(subscriber, payment_info.name);
            //  Cancel User Membership
        }
        Self::deposit_event(event_cancelled);

        Ok(())
    }
    pub(super) fn force_default(
        maybe_owner: Option<T::AccountId>,
        payment_id: PaymentIndex, 
        name: Vec<u8>,
        new_owner: T::AccountId,
        min_balance: T::Balance,
        frequency: Frequency,
        freezer: <T::Lookup as StaticLookup>::Source,
        is_frozen: bool,
        schedule_periodic_collection: Frequency,
        event_edit: Event<T>,
    ) -> DispatchResult {
        ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
        let mut payment_info = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;
        
        let bounded_name: BoundedVec<u8, T::StringLimit> =
			name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;


        PaymentInfo::<T>::try_mutate(payment_id, |info| { 
            let mut curr = info.take().ok_or(Error::<T>::Unknown)?;
            curr.merchant = T::Lookup::lookup(new_owner)?;
            curr.name = bounded_name;
            curr.required_payment = min_balance;
            curr.frequency = frequency;
            curr.subscribers = Zero::zero();
            curr.freezer = T::Lookup::lookup(freezer)?;
            curr.schedule_periodic_collection = schedule_periodic_collection;
            curr.is_frozen = is_frozen;
            *info = Some(curr);

            Self::deposit_event(event_edit);
            Ok(())
        });
    }
    pub(super) fn refund_to_users(
        maybe_owner: Option<T::AccountId>,
        payment_id: PaymentIndex,
    ) -> DispatchResult { 
        ensure!(!PaymentInfo::<T>::contains_key(&payment_id), Error::<T>::PaymentPlanDoesNotExist);
        let mut refunded = 0;
        let mut payment_info = PaymentInfo::<T>::get(&payment_id).ok_or(Error::<T>::Unknown)?;
        // Return all user funds 
        let subscriptions = Self::trie_iterator(&payment_id);
        for (user, (balance, _)) in subscriptions { 
            T::Currency::tranfer(&Self::fund_account_id(&payment_id),
                &user, 
                balance, 
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath
            )?;
            payment_info.total_deposits = payment_info.total_deposits.saturating_sub(balance);
            
            refunded += 1;
        }
        PaymentInfo::<T>::insert(&payment_id, payment_info);

        if refunded == payment_info.num_subscribers { 
            Self::deposit_event(Event::RefundedUsers { 
                sender: maybe_owner,
                id: payment_id, 
                now: T::Moment::now(),
            })
        } else { 
            Self::deposit_event(Event::<T>::PartiallyRefunded { 
                sender: maybe_owner,
                id: payment_id, 
                now: T::Moment::now(),
            })
        }
        Ok(())
    }
    fn calculate_leftover() {}
    fn calculate_next_duedate() {}
}
