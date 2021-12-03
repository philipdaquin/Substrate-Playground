#![cfg_attr(not(feature = "std"), no_std)]
//	A pallet that demonstrates the fundamentals of Fixed Point Arithmetic 
//	This pallet implements 3 Multiplicative accumulators using fixed point 

//	1. Manual Implementation -
//	2. PerMill Implmentation - ie. Saturating Mul
//	3. Substrate-fixed Implementation - trancendental functions 
pub use pallet::*;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};	
	use frame_system::pallet_prelude::*;
	use sp_arithmetic::{traits::Saturating, Permill};
	use substrate_fixed::types::U16F16;
	

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	//* Permill */
	#[pallet::type_value]
	pub(super) fn PermillAccumulatorDefaultValue<T: Config>() -> Permill { 
		Permill::one()
	}
	#[pallet::storage]
	#[pallet::getter(fn permill_value)]
	pub type PermillAccumulator<T: Config> = StorageValue<_, Permill, ValueQuery, PermillAccumulatorDefaultValue<T>>;
	
	//* Substrate Fixed Points */
	#[pallet::type_value]
	pub(super) fn FixedAccumulatorDefaultValue<T: Config>() -> U16F16 { 
		U16F16::from_num(1)
	}
	#[pallet::storage]
	#[pallet::getter(fn fixed_value)]
	pub(super) type FixedAccumulator<T: Config> =nStorageValue<_, U16F16, ValueQuery, FixedAccumulatorDefaultValue<T>>;

	//* Manual Implementation */
	#[pallet::type_value]
	pub fn ManualAccumulatorDefaultValue<T: Config>() -> u32 { 
		1 << 16
	}
	#[pallet::storage]
	#[pallet::getter(fn manual_value)]
	pub(super) type ManualAccumulator<T: Config> = StorageValue<_, u32, ValueQuery, ManualAccumulatorDefaultValue>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		//	The contained data is (new data, new product)
		PermillUpdated(Permill, Permill),
		//	Substrate fixed accumulator has been updated 
		FixedUpdated(U16F16, U16F16),
		//	Manual accumulator has been updated 
		ManualUpdated(u32, u32)
	}
	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_0000)]
		pub fn update_permill(
			origin: OriginFor<T>,
			new_factor: Permill
		) -> DispatchResult { 
			ensure_signed(origin)?;

			let old_accumulated = Self::permill_value();
			//	There is no need to check for overflow in Permill since it holds vlaues in the range of 0,1
			//	So it is impossible to ever overflow 
			let new_value = old_accumulated.saturating_mul(new_factor);
			PermillAccumulator::<T>::put(new_value);
			Self::deposit_event(Event::PermillUpdated(new_value, new_factor));
			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn updated_fixed(
			origin: OriginFor<T>,
			new_factor: U16F16
		) -> DispatchResultWithPostInfo { 
			ensure_signed(origin)?;
			let old_accumulated = Self::fixed_value();
			let new_value = old_accumulated.checked_add(&new_factor).ok_or(Error::<T>::StorageOverflow)?;
			FixedAccumulator::<T>::put(new_value);
			Self::deposit_event(Event::FixedUpdated(new_factor, new_value));


			Ok(().into())
		}
		#[pallet::weight(10_000)]
		pub fn update_manual(
			origin: OriginFor<T>,
			new_factor: u32
		) -> DispatchResultWithPostInfo { 
			ensure_signed(origin)?;
			//	To ensure we dont overflow unncessarily, the values are cast up to u64 before multiplyigng 
			//	This intermediate format has 48 integer positions and 16 fractional
			let old_accumulated: u64 = Self::manual_value() as u64;
			let new_factor_u64: u64 = new_factor as u64;

			//	Perform the multiplication on the u64 values 
			let raw_product: u64 = old_accumulated * new_factor_u64;

			//	Right shift to restore the convention that 15 bits are fractional
			let shifted_product: u64 = raw_product >> 16;

			//	Ensure that the product fits inu32, and effort if it doesn t
			if shifted_product > (u32::max_value() as u64) { 
				return Err(Error::<T>::StorageOverflow.into())
			}
			
			let final_product = shifted_product as u32;
			ManualAccumulator::<T>::put(final_product);
			Self::deposit_event(Event::ManualUpdated(new_factor, final_product));
			

			Ok(().into())
		}
	}
}
