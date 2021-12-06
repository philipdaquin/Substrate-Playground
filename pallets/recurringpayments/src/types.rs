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

use codec::{Encode, Decode, MaxEncodedLen};
use frame_support::{RuntimeDebug, traits::OnTimestampSet};
use scale_info::TypeInfo;
use crate::{PaymentIndex, Balance};

pub type BlockNumber = u32;
pub type Moment = u64;

pub const MILLISECS_PER_BLOCK: Moment = 3000;
pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;
pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;
pub const WEEKLY: BlockNumber = DAYS * 7;
pub const FORTHNIGHT: BlockNumber = WEEKLY * 2;
pub const MONTHLY: BlockNumber = WEEKLY * 4;
pub const YEARLY: BlockNumber = MONTHLY * 12;
pub const NONE: BlockNumber = 0;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum Frequency { 
	None,
	Daily,
	Monthly,
	Weekly,
	Forthnightly,
	Yearly
}
impl Frequency { 
	pub fn frequency(&self) -> u64 { 
		match self { 
			Frequency::Daily => DAYS,
			Frequency::Monthly => MONTHLY,
			Frequency::Weekly => WEEKLY,
			Frequency::Forthnightly => FORTHNIGHT,
			Frequency::Yearly => YEARLY,
			Frequency::None => NONE
		}
	}
}
// 	Identify the fund created by the owner
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct PaymentPlan<AccountId, Balance> { 
	//	The account that will receive the payments after the agreed contract has finsihed 
	pub merchant: AccountId,
	//	plan name  
	pub name: Vec<u8>,
	//	payment_id 
	pub payment_id: PaymentIndex,
	//	The minimum deposit required to subcribe to become a member
	pub required_payment: Balance,
	//	The total amount of deposit generated by the service/ product
	pub total_deposits: Balance,
	//	Frequency of payment 
	pub frequency: Frequency,
	//	The number of subscreibers this payment plan current have 
	pub num_subscribers: u32,
	//	Can freeze the payment inflows 
	pub freezer: AccountId,
	//	Whether the asset is frozen for non-admin transfer
	pub is_frozen: bool,  
	//	> Number of subscribers currently 

}
//	This will help us track the user's next payment 
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Subscription<AccountId, Moment, Balance> { 
	//	The subscriber for payment 
	pub owner: AccountId,
	//	start date
	pub start: Moment,
	//	How much this subscription is costing this account
	pub required_payment: Balance,
	//	Next Due Paymetn 
	pub next_payment: Moment, 
	//	Time to be Charged
	pub frequency_of: Frequency,
	//	How many times the user wants to keep recurring payment on
	pub num_frequency: Option<u32>,
	//	Specify which you are subscribed to payment_plan
	pub subscribed_to: Vec<u8>

}
//	Custom Methods for Subscruption 
//	- Calculate the next payment based on start date and frequency of payment plan 
impl<AccountId, Moment, Balance> Subscription<AccountId, Moment, Balance> { 
	//	Next payment = Start Date + Frequency in terms of (Monthly , Daily, Monthly etc)
	pub fn schedule_next_payment(mut self, start: Moment) -> Self { 
		self.next_payment = self.start + self.frequency_of;
		self
	}
} 
//	Custom Methods for Payment Plans
//	- Get frequency of PaymentPlan
impl<AccountId, Moment> PaymentPlan<AccountId, Moment> { 
	pub fn frequency(&self) -> Self { 
		self.frequency
	}
}

pub const AQUARTER: u8 = 25;
pub const HALF: u8 = 50;
pub const THREEQUARTERS: u8 = 50;
pub const FULL: u8 = 100;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum Portion { 
	AQuarter,
	Half,
	ThreeQuarters,
	Full,
}

impl Portion { 
	pub fn portion(&self) -> u128 { 
		match self { 
			Portion::AQuarter => AQUARTER,
			Portion::Half => HALF,
			Portion::ThreeQuarters => THREEQUARTERS,
			Portion::Full => FULL
		}
	}
}