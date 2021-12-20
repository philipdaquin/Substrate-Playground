


pub struct Price<Moment, Balance, CurrencyId> {
	pub id: PriceId,
	pub object: Object,
	pub billing_scheme: BillingScheme,
	#[codec(compact)]
	pub created_at: Moment,
	pub currency: CurrencyId,
	pub livemode: bool,
	description: Vec<u8>,
	pub product: ProductId, 
	//pub recurring: Option<Recurring>, 
	pub tiers_mode: Option<TiersMode>,
	pub purchase_type: Type,
	// Represent how much to charge:
	#[codec(compact)]
	pub unit_amount: Balance,
	pub unit_amount_decimal: Option<Decimal>,
}

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
pub enum Type<Balance> {
	OneTime(Balance),
	Recurring {
		aggregated_usage: UsageTypes,
		interval: Interval,
		interval_count: u32,
		usage_type: UsageTypes
	}
}