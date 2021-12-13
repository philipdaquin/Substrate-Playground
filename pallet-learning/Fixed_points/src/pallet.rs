//use super::*;
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
	pub(super) type FixedAccumulator<T: Config> = StorageValue<_, U16F16, ValueQuery, FixedAccumulatorDefaultValue<T>>;

	//* Manual Implementation */

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
			



			Ok(().into())
		}

	}
