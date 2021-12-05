

use crate::{types::*, Balance};
use crate::{PaymentIndex};
use frame_support::sp_std::prelude::*;
use sp_runtime::traits::Zero;

#[derive(Default)]
pub struct PaymentPlanBuilder<AccountId, Balance>
	where AccountId: Default, Balance: Default
{ 
	merchant: AccountId,
	name: Vec<u8>,
	payment_id: PaymentIndex,
	required_payment: Balance,
	total_deposits: Balance,
	frequency: Frequency

}
//	Set up default values for payment plans -> Set to default values 
impl<AccountId, Balance> Default for PaymentPlanBuilder<AccountId, Balance> 
	where AccountId: Default, Balance: Default
{ 
	fn default() -> Self { 
		PaymentPlanBuilder { 
			merchant: AccountId::default(),
			name: Self::default(),
			payment_id: PaymentIndex::default(),
			required_payment: Zero::zero(),
			total_deposits: Zero::zero(),
			frequency: Frequency::None
		}
	}
}

impl<AccountId, Balance> PaymentPlanBuilder<AccountId, Balance> 
	where AccountId: Default, Balance: Default
{
	pub fn identified_by(mut self, payment_id: PaymentIndex) -> Self { 
		self.payment_id = payment_id;
		self
	}
	pub fn owned_by(mut self, merchant: AccountId) -> Self { 
		self.merchant = merchant;
		self
	}
	pub fn with_name(mut self, name: Vec<u8>) -> Self { 
		self.name = name;
		self
	}
	pub fn min_payment(mut self, required_payment: Balance) -> Self { 
		self.required_payment = required_payment;
		self
	}
	pub fn new_deposit(mut self, total_deposits: Balance) -> Self { 
		self.total_deposits = self.total_deposits + total_deposits;
		self
	}
	pub fn payment_frequency(mut self, frequency: Frequency) -> Self { 
		self.frequency = frequency;
		self
	}
	pub fn build(self) -> PaymentPlan<AccountId, Balance> { 
		PaymentPlan::<AccountId, Balance> { 
			merchant: self.merchant,
			name: self.name,
			payment_id: self.payment_id,
			required_payment: self.required_payment,
			total_deposits: self.total_deposits,
			frequency: self.frequency
		}
	}
}
#[derive(Default)]
pub struct SubscriptionBuilder<AccountId, Moment> 
	where AccountId: Default, Moment: Default
{
	owner: AccountId,
	start: Moment, 
	next_payment: Moment, 
	frequency_of: Frequency
}
impl<AccountId, Moment> SubscriptionBuilder<AccountId, Moment> 
	where AccountId: Default, Moment: Default
{ 
	pub fn account_id(mut self, owner: AccountId) -> Self { 
		self.owner = owner;
		self
	}
	pub fn start_date(mut self, start: Moment) -> Self { 
		self.start = start;
		self
	}
	pub fn next_payment(mut self, next_payment: Moment) -> Self { 
		self.next_payment = next_payment;
		self
	}
	pub fn frequency_type(mut self, frequency_of: Frequency) -> Self{ 
		self.frequency_of = frequency_of;
		self
	}
	pub fn build(self) -> Subscription<AccountId, Moment> { 
		Subscription::<AccountId, Moment> { 
			owner: self.owner,
			start: self.start,
			next_payment: self.next_payment,
			frequency_of: self.frequency_of
		}
	}
}