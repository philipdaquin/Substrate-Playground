
use super::*;
//  Prices define the unit cost, currency and billing cycle for 
//  recurring and one time purchases of products 
#[derive(Encode, Decode, Default, Debug, Eq, PartialEq, Clone, PartialOrd, Ord)]
pub struct Price<Moment, BalanceOf, CurrencyId> {
    // Identifier for associated for Price/ billing 
	pub id: PriceId,
    //  Identifier type 
	pub object: Object,
	//	Whether the price can be used for new purchases. Defaults to true 
	pub active: bool,
	//  Created by who, and when 
	pub created_by: CreatedBy<T>,
    //  Describes how to compute the price per period, can either be: Per_unit or Tiered
    //  'Per Unit' indicates that the fixed amount will be charged per unit in quantity (for plans == licencesed)
    pub billing_scheme: BillingScheme,
	//	Time at which the object was created. Measured in seconds in UNIX
    #[codec(compact)]
	pub created_at: Moment,
	//	Must be supported Currency 
    pub currency: CurrencyId,
	//	Has the value of true if the object exists in live modeor the value false if the object 
	//	exists in test mode
    pub livemode: bool,
    // An arbitrary string attached to the object. Often useful for displauing to users
	pub description: Vec<u8>,
	//	Id of the product this price is associated with
    pub product: ProductId, 
	//pub recurring: Option<Recurring>, 
    pub tiers_mode: Option<TiersMode<DepositBalance, Balance>>,
    //	Type: One of OneTime or Recurring depending on whether the price is for a one-time purchase or a recurring purchase
	pub purchase_type: Type,
	// Represent how much to charge:
    #[codec(compact)]
	pub unit_amount: Balance,
	//	Amount in cents
    pub unit_amount_decimal: Option<Decimal>,
}
//  Represented as BlockNumbers
pub enum Interval {
	Month,
	Year,
	Week,
	Day
}
pub enum UsageTypes {
	Metered { 
        total_sum_of_usage: Option<Balance>,
        last_during_period: Option<Balance>,
        last_ever: Option<Balance>
    },
	Licensed
}
pub enum TiersMode<DepositBalance, Balance> {
	Graduated {
		next_first_unit: u32,
		next_last_unit: u32,
		per_unit: Balance,
		flat_fee: DepositBalance
		
	},
	Volume {
		total_first_unit: u32,
		total_last_unit: u32,
		per_unit: Balance,
		flat_fee: DepositBalance
		}
}
pub enum BillingScheme {
	Per_unit,
	Tiered
}
pub enum Type<Balance, BlockNumber> {
	OneTime(Balance),
	Recurring {
		aggregated_usage: UsageTypes,
        interval: Interval,
		interval_as_blocknumber: BlockNumber,
		interval_count: u32,
		usage_type: UsageTypes
	}
}

pub struct CreatedBy<T: Config> { 
	pub account_id: T::AccountId,
	pub blocknumber: T::BlockNumber,
	pub time: T::Moment,
}