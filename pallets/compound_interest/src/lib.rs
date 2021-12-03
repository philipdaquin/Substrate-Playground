#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

//	A pallet that demonstrates fixed point airthmetic in the context of two simple bank accounts that accrue compounding interest

use scale_info::TypeInfo;
use sp_arithmetic::Percent;
use sp_std::convert::TryInto;
use codec::{Encode, Decode};
pub use pallet::*;
use substrate_fixed::{transcendental::exp, types::I32F32};
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[derive(Encode, Decode, Default, Clone, TypeInfo)]
pub struct AccountData<BlockNumber> {
	//	The balance of the account after last manual adjustment 
	pub principal: I32F32,
	//	The time at which teh balanace was last adjusted 
	pub deposit_date: BlockNumber
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{dispatch::{DispatchResult, DispatchResultWithPostInfo}, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Zero;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	#[pallet::getter(fn balance_compound)]
	pub(super) type Accounts<T: Config> = StorageValue<_, AccountData<T::BlockNumber>, ValueQuery>;


	//	Stored inside is the interest amount applied to DiscreteAccount (not the new or old balance) 
	#[pallet::storage]
	#[pallet::getter(fn discrete_account)]
	pub(super) type DiscreteAccount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		//	Deposited some balance into the compoudning interest account
		DepositedContinuous(u64),
		//	Withdrew some balance from the compounding interest account 
		WithdrewContinuous(u64),
		//	Desposited some balance into the discrete interest account 
		DepositedDiscrete(u64),
		//	Withdrew some balance from the discrete interest account 
		WithdrewDiscrete(u64),
		//	This holds the interest amount (not the new or old balance)
		DiscreteInterestApplied(u64)
	}

	//	Apply newly accrued discrete interest every ten blocks
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> { 
		fn on_finalize(n: T::BlockNumber) { 

			if (n % 10u32.into()).is_zero() { 
				//	Calculate the interest 
				//	Interest = Principal * Rate * Time
				let interest = Self::discrete_interest_rate() * DiscreteAccount::<T>::get() * 10;

				let old_balance = DiscreteAccount::<T>::get();
				DiscreteAccount::<T>::put(old_balance + interest);

				Self::deposit_event(Event::DiscreteInterestApplied(interest));
			}
		}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		U32ConversionError
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn deposit_continuous(
			origin:OriginFor<T>,
			val: u64
		) -> DispatchResultWithPostInfo { 
			let sender = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();
			let old_value = Self::value_continuous(&now);
			//	Update Storge for Compounding Account 
			Accounts::<T>::put( AccountData { 
				principal: old_value + I32F32::from_num(val),
				deposit_date: now
			});
			Self::deposit_event(Event::DepositedContinuous(val));
			Ok(().into())
		}
		#[pallet::weight(10_000)]
		pub fn withdraw_continuous(
			origin: OriginFor<T>,
			val: u64 
		) -> DispatchResultWithPostInfo { 
			let sender = ensure_signed(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			// Get the current value of the compounding interest amount 
			let old_value = Self::value_continuous(&now);
			
			Accounts::<T>::put( AccountData { 
				principal: old_value - I32F32::from_num(val),
				deposit_date: now
			});
			Self::deposit_event(Event::WithdrewContinuous(val));

			Ok(().into())
		}
		#[pallet::weight(10_000)]
		pub fn deposit_discrete(
			origin: OriginFor<T>,
			val: u64
		) -> DispatchResultWithPostInfo { 
			let sender = ensure_signed(origin)?;

			let old_val = DiscreteAccount::<T>::get();
			DiscreteAccount::<T>::put(val + old_val);
			Self::deposit_event(Event::DepositedDiscrete(val));
			Ok(().into())
		}
		#[pallet::weight(10_000)]
		pub fn withdraw_discrete(
			origin: OriginFor<T>,
			val: u64
		) -> DispatchResultWithPostInfo { 
			let sender = ensure_signed(origin)?;

			let old_val = DiscreteAccount::<T>::get();
			DiscreteAccount::<T>::put(old_val + val);
			Self::deposit_event(Event::WithdrewDiscrete(val));
			Ok(().into())
		}
	}
	impl<T: Config> Pallet<T> { 
		//	Hard coded 5% interest rate 
		fn discrete_interest_rate() -> Percent { 
			Percent::from_percent(6)
		}
		//	Evaluate the current value of the continously compounding interest account 
		//	"How much is to be gained during compounding period"
		fn value_continuous(now: &<T as frame_system::Config>::BlockNumber) -> I32F32 { 
			//	Get the old state of the account 
			let AccountData { 
				principal, 
				deposit_date,
			} = Accounts::<T>::get();

			//	Convert BlockNumber to Fixed I64
			//	Calculate the exponential function 
			let elapsed_time_blocknumber = *now - deposit_date;
			//	BlockNumber into U32
			let elapsed_time_into_u32: u32 = TryInto::try_into(elapsed_time_blocknumber)
				.ok()
				.expect("Blockchain will not Exceed 2^32 blocks; qed");
			//	U32 into i32f32 (I64)
			let elapsed_time_i32f32 = I32F32::from_num(elapsed_time_into_u32);
			//	Interest rate * Time in I32F32 
			let exponent = Self::continuous_interest_rate() * elapsed_time_i32f32;
			//	Reuslt in expoential function 
			let exp_result: I32F32 = exp(exponent).expect("Interest Will Not Overflow Account");
			
			principal * exp_result
		}
		//	Return the Hard coded 5% interest rate
		//	3/50 = 6% =>  The same interest rate as the discrete account, but in the Substrate Fixed I64
		fn continuous_interest_rate() -> I32F32 { 
			I32F32::from_num(3)/50
		}
	}
}
