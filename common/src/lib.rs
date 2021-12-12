use scale_info::TypeInfo;
use sp_core::{
	u32_trait::{_1, _2, _3, _4},
	H160,
};
use sp_runtime::{
	traits::{BlockNumberProvider, Convert},
	transaction_validity::TransactionPriority,
	Perbill,
};
use static_assertions::const_assert;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Get;
use frame_support::{
	parameter_types,
	traits::Contains,
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, WEIGHT_PER_MILLIS, WEIGHT_PER_NANOS},
		DispatchClass, Weight,
	},
	RuntimeDebug,
};
use frame_system::{limits, EnsureOneOf, EnsureRoot};
pub use module_support::{ExchangeRate, PrecompileCallerFilter, Price, Rate, Ratio};


/// The call is allowed only if caller is a system contract.
pub struct SystemContractsFilter;
impl PrecompileCallerFilter for SystemContractsFilter {
	fn is_allowed(caller: H160) -> bool {
		is_system_contract(caller)
	}
}

// TODO: estimate this from benchmarks
// total block weight is 500ms, normal tx have 70% of weight = 350ms
// 350ms / 25ns = 14M gas per block
pub const WEIGHT_PER_GAS: u64 = 25 * WEIGHT_PER_NANOS; // 25_000

/// Convert gas to weight
pub struct GasToWeight;
impl Convert<u64, Weight> for GasToWeight {
	fn convert(gas: u64) -> Weight {
		gas.saturating_mul(WEIGHT_PER_GAS)
	}
}

// TODO: somehow estimate this value. Start from a conservative value.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// The ratio that `Normal` extrinsics should occupy. Start from a conservative value.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(70);
/// Parachain only have 0.5 second of computation time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = 500 * WEIGHT_PER_MILLIS;

const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());

parameter_types! {
	/// Maximum length of block. Up to 5MB.
	pub RuntimeBlockLength: limits::BlockLength =
		limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	/// Block weights base values and limits.
	pub RuntimeBlockWeights: limits::BlockWeights = limits::BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have an extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT,
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
}

parameter_types! {
	/// A limit for off-chain phragmen unsigned solution submission.
	///
	/// We want to keep it as high as possible, but can't risk having it reject,
	/// so we always subtract the base block execution weight.
	pub OffchainSolutionWeightLimit: Weight = RuntimeBlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic
		.expect("Normal extrinsics have weight limit configured by default; qed")
		.saturating_sub(BlockExecutionWeight::get());
}

pub struct DummyNomineeFilter;
impl<AccountId> Contains<AccountId> for DummyNomineeFilter {
	fn contains(_: &AccountId) -> bool {
		true
	}
}


pub fn dollar(currency_id: CurrencyId) -> Balance {
	10u128.saturating_pow(currency_id.decimals().expect("Not support Non-Token decimals").into())
}

pub fn cent(currency_id: CurrencyId) -> Balance {
	dollar(currency_id) / 100
}

pub fn millicent(currency_id: CurrencyId) -> Balance {
	cent(currency_id) / 1000
}

pub fn microcent(currency_id: CurrencyId) -> Balance {
	millicent(currency_id) / 1000
}

/// The type used to represent the kinds of proxying allowed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ProxyType {
	Any,
	CancelProxy,
	Governance,
	Auction,
	Swap,
	Loan,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
