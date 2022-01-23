#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::{Randomness, UnixTime}};
use frame_system::{pallet_prelude::*, RawOrigin};
use orml_currencies::Currency;


mod types;
use crate::types::*;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;
// #[cfg(feature = "runtime-benchmarks")]

pub mod functions;
#[frame_support::pallet]
pub mod pallet {
	use sp_io::hashing::blake2_128;

	use super::*;
	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// Used to represent Currency Id and Balances 
		type Currency: MultiReservableCurrency<Self::AccountId>;
		// Used to generate Price Id
		type IdRandomness: Randomness<Self::Hash, Self::BlockNumber>;
		// Used to limit Strings
		type StringLimit: Get<u32>;
		// Origin from which the Prices are set at
		type Merchant: EnsureOrigin<Self::Origin>;
		//	UnixTime
		type UnixTime: UnixTime;
		// The basic amount of funds that must be reserved for the transaction
		type FlatFee: Get<DepositBalanceOf<Self>>;
		// Recurring Intervals:
		//	As Specified on Runtime, Intervals are measured as blocks
		type Month: Get<Self::BlockNumber>;
		//	YEARS: BlockNumber = MONTHS * 12;
		type Year: Get<Self::BlockNumber>;
		//	 WEEKS: BlockNumber = DAYS * 7;
		type Week: Get<Self::BlockNumber>;
		//	DAYS: BlockNumber = HOURS * 24;
		type Day: Get<Self::BlockNumber>;
	}
	//	Identifier
	pub type PriceId = [u8; 16];
	// Used to represent Prices 
	pub type BalanceOf<T> = <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	// used to represent Currency Ids
	pub type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
	//	Deposit Balance for Graduate/ Volume pricing models 
	pub type DepositBalanceOf<T> =
	<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10000)]
		pub fn create_price(
			origin: OriginFor<T>,
		) -> DispatchResult { 

			Ok(())
		}




	}
	
}
