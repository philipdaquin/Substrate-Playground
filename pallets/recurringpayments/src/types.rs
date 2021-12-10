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
use sp_arithmetic::Percent;
use codec::{Encode, Decode, MaxEncodedLen};
use frame_support::{RuntimeDebug, traits::OnTimestampSet};
use scale_info::TypeInfo;
pub type BlockNumber = u32;
pub type Moment = u64;

pub const MILLISECS_PER_BLOCK: Moment = 6000;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum Frequency { 
	None,
	Daily,
	Monthly,
	Weekly,
	Forthnightly,
	Yearly
}

//	Due Date is determined by Block Per MilliSecs. DueDate = StartingBlock + Interval Based on Frequency
//	milli per block =	12000 milli seconds
// 	sec	= 12 Blocknumber
// 	min	= 5 Bloclnumber 
// 	hour =	300
// 	day	= 7200
// 	week	= 50400
// 	monthly	= 201600
// 	yearly 	= 2419200
impl Frequency { 
	pub fn frequency(&self) -> BlockNumber { 
		
		let mut secs = MILLISECS_PER_BLOCK / 1000;
		let mut min: BlockNumber = 60 / (secs as BlockNumber);
		let mut hour: BlockNumber = min * 60;
		let mut days: BlockNumber = hour * 24;
		let mut week: BlockNumber = days * 7;
		let mut month: BlockNumber = week * 4;
		let mut year: BlockNumber = month * 12;
		
		match self { 
			Frequency::Daily => days,
			Frequency::Monthly => month,
			Frequency::Weekly => week,
			Frequency::Forthnightly => week/2,
			Frequency::Yearly => year,
			Frequency::None => 0 as BlockNumber
		}
	}
}


// 	Identify the fund created by the owner
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
pub struct PaymentPlan<AccountId, BalanceOf, BoundedString> { 
	//	The account that will receive the payments after the agreed contract has finsihed 
	pub merchant: AccountId,
	//	plan name  
	pub name: BoundedString,
	//	payment_id 
	pub payment_id: PaymentIndex,
	//	The minimum deposit required to subcribe to become a member
	pub required_payment: BalanceOf,
	//	The total amount of deposit generated by the service/ product
	pub total_deposits: BalanceOf,
	//	Frequency of payment 
	pub frequency: Frequency,
	//	The number of subscreibers this payment plan current have 
	pub num_subscribers: u32,
	//	Can freeze the payment inflows 
	pub freezer: Option<AccountId>,
	//	Whether the asset is frozen for non-admin transfer
	pub is_frozen: bool,  
	//	> Number of subscribers currently 
	pub schedule_periodic_collection: Option<Frequency>,
	

}
//	This will help us track the user's next payment 
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
pub struct Subscription<AccountId, BlockNumber, BalanceOf> { 
	pub id: SubscriptionIndex, 
	//	The subscriber for payment 
	pub owner: AccountId,
	//	start date
	pub start: BlockNumber,
	//	How much this subscription is costing this account
	pub required_payment: BalanceOf,
	//	Next Due Paymetn 
	pub next_payment: BlockNumber, 
	//	Time to be Charged
	pub frequency_of: Frequency,
	//	How many times the user wants to keep recurring payment on
	pub num_frequency: Option<u32>,
	//	Specify which you are subscribed to payment_plan, paymentIndexes are u32
	pub subscribed_to: Vec<u32>

}

pub const AQUARTER: Percent = Percent::from_percent(25);
pub const HALF: Percent = Percent::from_percent(50);
pub const THREEQUARTERS: Percent = Percent::from_percent(75);
pub const FULL: Percent = Percent::from_percent(100);

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum Portion { 
	AQuarter,
	Half,
	ThreeQuarters,
	Full,
}

impl Portion { 
	pub fn portion(&self) -> Percent { 
		match self { 
			Portion::AQuarter => AQUARTER,
			Portion::Half => HALF,
			Portion::ThreeQuarters => THREEQUARTERS,
			Portion::Full => FULL
		}
	}
}

