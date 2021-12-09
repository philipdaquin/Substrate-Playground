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

//! Functions for the Permissioned Membership pallet.

use codec::{Encode, Decode};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

use super::*;
//  Executors can trigger dispatch functions
#[derive(Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Permissions { 
    Management,
    Executors, 
}
//  Executors are your Buyers and Sellers
#[derive(Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Executors { 
    Seller,
    Buyer, 
}

impl Permissions { 
    pub(super) fn as_bytes(&self) -> &[u8] { 
        match self { 
            Permissions::Management => b"Permissions::Management",
            Permissions::Executors => b"Permissions::Executors",
        }
    }
} 

//  Default values for membership is either an buyer or a seller 
impl Default for Permissions { 
    fn default() -> Self { 
        Permissions::Executors
    }
}

//  Permissions will set the the user's contraints in other pallets 
#[derive(Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Role {
    pub pallet_name: Vec<u8>,
    pub permission: Permissions
}