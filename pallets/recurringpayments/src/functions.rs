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
use frame_system::pallet_prelude::*;
use frame_support::storage::ChildTriePrefixIterator;
use sp_runtime::traits::StaticLookup;
use frame_support::{ensure, traits::Get};
use frame_support::{ BoundedVec};

pub(super) use pallet::*;
use super::*;
// use pallet::{Store, PaymentInfo, SubscriptionId, SubscriptionIndex, Subscriptions, Pallet, };
// use crate::types::*;
// use crate::builder::*;

