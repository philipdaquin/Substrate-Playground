#![cfg_attr(not(feature = "std"), no_std)]
use codec::Codec;
use frame_support::pallet_prelude::{Member, MaybeSerializeDeserialize};
//	Substrate MarketPlace including Buyer and Seller Reputation
//	Uses: Ride Sharing rider and Driver profile 
pub use pallet::*;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


//	Reputation Trait 
pub trait Reputation<AccountId, Moment> { 
	//	The reputation of the vendor 
	type Reputation_Score;
	type Feedback: Member + Codec + MaybeSerializeDeserialize;
	//	Allow an account to give ratings to the vendor 
	fn rate_seller(
		buyer: AccountId,
		seller: AccountId, 
		comment: Self::Feedback, 
		now: Moment

	) -> Result<(), Error<()>>;
	//	Assigns a rating to another person 
	fn rate_buyer(
		buyer: AccountId,
		seller: AccountId, 
		comment: Self::Feedback,
		now: Moment
		
	) -> Result<(), Error<()>>;
	//	The current ratings of an account 
	fn reputation(sender: AccountId) -> Self::Reputation_Score;
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Moment: UnixTime;
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

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {


	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

	} 
	impl<T: Config> Pallet<T> { 

	}
}
