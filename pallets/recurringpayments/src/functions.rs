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
    fn do_create() {}
    fn do_collect() {}
    fn do_transfer_ownership() {}
    fn do_subscribed() {}
    fn do_force_pay() {}
    fn calculate_leftover() {}
}
