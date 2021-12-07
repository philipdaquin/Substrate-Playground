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
        //  Schedule Periodic Collections
        


        Self::deposit_event(event);
        Ok(())

    }
    pub(super) fn do_collect_payments(
        payment_id: PaymentIndex,
        
    ) -> DispatchResult {

        Ok(())
    }
    fn do_transfer_ownership() {}
    
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
        Self::deposit_event(event_sub_created);

        Ok(())
    }
    fn do_force_pay() {}
    fn calculate_leftover() {}
    fn calculate_next_duedate() {}
}
