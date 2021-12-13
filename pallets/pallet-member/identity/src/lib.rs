//! Decentralised Identifiers
//! A new type of identifier that enables verifiable, decentralised digital identity
//! DIDs have been designed so that they may be decouled from centralised registries, 
//! identity providers and certificate authorities
//! 
//! DIDs are URls that associate a DID subject with a DID document allowing trustable interactions 
//! associated with that subject 
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
mod types;
mod traits;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use codec::{Encode, Decode};
use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
	


#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::Time;
use sp_core::blake2_256;
use sp_runtime::traits::{IdentifyAccount, Verify};

use crate::{types::Attribute, traits::Identity};

use super::*;
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		//	A type that will collapsed into an account Id
		type Public: IdentifyAccount<AccountId = Self::AccountId>;
		//	Signature Verification 
		type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
		type Time: Time;
	}

	pub type Moment<T> = <<T as Config>::Time as Time>::Moment;
	pub type BlockNumber<T> = <T as frame_system::Config>::BlockNumber;
	pub type AccountId<T> = <T as frame_system::Config>::AccountId;
	//	Identity delegates stored by type 
	//	Delegates are only valud for a specific period defined as blocks number 
	
	
	
	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type DelegateOf<T> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, Vec<u8>, T::AccountId),
		Option<BlockNumber<T>>,
		ValueQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn attribute_of)]
	pub type AttributeOf<T> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, [u8; 32]),
		Attribute<T::BlockNumber, Moment<T>>,
		ValueQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn nonce_of)]
	pub type AttributeNonce<T> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, Vec<u8>),
		u64,
		ValueQuery
	>;
	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub type OwnerOf<T> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, 
		Option<T::AccountId>,
		ValueQuery
	>;
	#[pallet::storage]
	#[pallet::getter(fn updated_by)]
	pub type UpdatedBy<T> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		(T::AccountId, T::BlockNumber, Moment<T>),
		ValueQuery
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {


	}

	#[pallet::error]
	pub enum Error<T> {
		NotOwner,
        InvalidDelegate,
        BadSignature,
        AttributeCreationFailed,
        AttributeResetFailed,
        AttributeRemovalFailed,
        InvalidAttribute,
        Overflow,
        BadTransaction,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn change_owner(
			origin: OriginFor<T>,
			identity: T::AccountId,
			new_owner: T::AccountId
		) -> DispatchResult { 


			Ok(())
		}

		#[pallet::weight(0)]
		pub fn add_delegate(
			origin: OriginFor<T>
		) -> DispatchResult { 

			Ok(())
		}
		#[pallet::weight(0)]
		pub fn revoke_delegate(
			origin: OriginFor<T>
		) -> DispatchResult { 

			Ok(())
		}
		#[pallet::weight(0)]
		pub fn add_attribute(
			origin: OriginFor<T>
		) -> DispatchResult { 

			Ok(())
		}
		#[pallet::weight(0)]
		pub fn revoke_attribute(
			origin: OriginFor<T>
		) -> DispatchResult { 

			Ok(())
		}
		#[pallet::weight(0)]
		pub fn delete_attribute(
			origin: OriginFor<T>
		) -> DispatchResult { 

			Ok(())
		}
		#[pallet::weight(0)]
		pub fn execute(
			origin: OriginFor<T>
		) -> DispatchResult { 

			Ok(())
		}
	}
	//	Publiic Functions
	impl<T: Config> Pallet<T> { 

	}
}
