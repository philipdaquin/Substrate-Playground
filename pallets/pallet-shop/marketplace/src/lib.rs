#![cfg_attr(not(feature = "std"), no_std)]

///	Amazon Prime Service 
/// Dependencies: 
/// - Subscription Payment Service 
/// - Marketplace 
/// - Membership and Role Based 

pub use pallet::*;



#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {


	}

	#[pallet::error]
	pub enum Error<T> {


	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
	}
	//	Helper Functions
	impl<T: Config> Pallet<T> { 

	}
	// impl<T: Config> Reputation<T> for Pallet<T> { 

	// }
}
