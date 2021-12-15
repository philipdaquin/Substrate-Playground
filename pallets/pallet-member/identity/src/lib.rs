//! Decentralised Identifiers
//! A new type of identifier that enables verifiable, decentralised digital identity
//! DIDs have been designed so that they may be decouled from centralised registries, 
//! identity providers and certificate authorities
//! 
//! DIDs are URls that associate a DID subject with a DID document allowing trustable interactions 
//! associated with that subject 
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub mod types;
pub mod traits;
pub mod functions;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use codec::{Encode, Decode};
use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use frame_support::traits::Time;
use scale_info::TypeInfo;
use sp_io::hashing::blake2_256;

use sp_runtime::traits::{IdentifyAccount, Verify, Member};
use crate::traits::*;

#[frame_support::pallet]
	pub mod pallet {
	use scale_info::TypeInfo;
use sp_std::{prelude::*};
	use crate::types::AttributeTransaction;
use crate::types::Attribute;
use super::*;
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

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
	#[pallet::getter(fn delegate_of)]
	pub type DelegateOf<T> = StorageMap<
		_,
		Blake2_128Concat,
		(AccountId<T>, Vec<u8>, AccountId<T>),
		Option<BlockNumber<T>>,
		ValueQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn attribute_of)]
	pub type AttributeOf<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, [u8; 32]),
		Attribute<T::BlockNumber, <<T as Config>::Time as Time>::Moment>,
		ValueQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn nonce_of)]
	pub type AttributeNonce<T> = StorageMap<
		_,
		Blake2_128Concat,
		(AccountId<T>, Vec<u8>),
		u64,
		ValueQuery
	>;
	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub type OwnerOf<T> = StorageMap<
		_,
		Blake2_128Concat,
		AccountId<T>, 
		AccountId<T>,
		OptionQuery
	>;
	#[pallet::storage]
	#[pallet::getter(fn updated_by)]
	pub type UpdatedBy<T> = StorageMap<
		_,
		Blake2_128Concat,
		AccountId<T>,
		(AccountId<T>, BlockNumber<T>, <<T as Config>::Time as Time>::Moment),
		ValueQuery
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config>  {
		OwnerChanged { 
			identity: T::AccountId,
			owner: T::AccountId,
			new_owner: T::AccountId, 
			now: T::BlockNumber
		},
		DelegateAdded { 
			identity: T::AccountId,
			delegate_type: Vec<u8>,
			delegate: T::AccountId, 
			valid_for: Option<T::BlockNumber>
		},
		DelegateRevoked { 
			identity: T::AccountId,
			delegate_type: Vec<u8>,
			delegate: T::AccountId,
		},
		AttributeAdded { 
			identity: T::AccountId,
			name: Vec<u8>,
			valid_for: Option<T::BlockNumber>
		},
		AttributeRevoked { 
			identity: T::AccountId,
			name: Vec<u8>,
			now: T::BlockNumber
		},
		AttributeDeleted { 
			identity: T::AccountId,
			name: Vec<u8>, 
			now: T::BlockNumber,
			
		},
		AttributeTransactionExecuted(AttributeTransaction<T::Signature, AccountId<T>>)
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
		Unknown
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>  {
		///		Transfers an identity represented as an AccountId from the owner account to 
		/// 	to a target account 
		#[pallet::weight(0)]
		pub fn change_owner(
			origin: OriginFor<T>,
			identity: T::AccountId,
			new_owner: T::AccountId
		) -> DispatchResult { 
			//	Check if we have an identity account or check if we have owner
			let owner = ensure_signed(origin)?;
			Self::is_owner(&identity, &owner)?;
			if OwnerOf::<T>::contains_key(&identity) {
                // Update to new owner.
                OwnerOf::<T>::mutate(&identity, |o| *o = Some(new_owner.clone()));
            } else {
                // Add to new owner.
                OwnerOf::<T>::insert(&identity, &new_owner);
            }
			let now = frame_system::Pallet::<T>::block_number();
			UpdatedBy::<T>::insert(&identity, (&owner, &now, T::Time::now()));
			Self::deposit_event(Event::OwnerChanged { 
				identity,
				owner,
				new_owner, 
				now
			});

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn add_delegate(
			origin: OriginFor<T>,
			identity: T::AccountId,
			delegate: T::AccountId, 
			delegate_type: Vec<u8>,
			valid_for: Option<T::BlockNumber>
		) -> DispatchResult { 
			let owner = ensure_signed(origin)?;
			ensure!(delegate_type.len() <= 64, Error::<T>::InvalidDelegate);

			Self::create_delegate(&owner, &identity, &delegate, &delegate_type, valid_for)?;
			let now = frame_system::Pallet::<T>::block_number();
			let time = T::Time::now();
			
			UpdatedBy::<T>::insert(&identity, (&owner, &now, time));
			
			Self::deposit_event(Event::DelegateAdded { 
				identity, 
				delegate_type, 
				delegate,
				valid_for,
			});
			Ok(())
		}
		///		The process of assigning a specific identity attribute or feature 
		#[pallet::weight(0)]
		pub fn revoke_delegate(
			origin: OriginFor<T>,
			identity: T::AccountId, 
			delegate: T::AccountId,
			delegate_type: Vec<u8>
		) -> DispatchResult { 
			let owner = ensure_signed(origin)?;
			ensure!(delegate_type.len() <= 64, Error::<T>::InvalidDelegate);
			//	Check if the owner own the delegate
			Self::is_owner(&identity, &owner)?;
			//	Check if the delegate is valid 
			Self::valid_listed_delegate(&identity, &delegate_type, &delegate)?;
			let now = frame_system::Pallet::<T>::block_number();
			let time = T::Time::now();
			DelegateOf::<T>::mutate(
				(&identity, &delegate_type, &delegate), |block| 
				*block = Some(now)
			);

			//	Update Storage 
			UpdatedBy::<T>::insert(&identity, (owner, now, time));
			
			Self::deposit_event(Event::DelegateRevoked { 
				identity,
				delegate_type,
				delegate,
			});
			Ok(())
		}
		#[pallet::weight(0)]
		pub fn add_attribute(
			origin: OriginFor<T>,
			identity: T::AccountId,
			name: Vec<u8>,
			value: Vec<u8>,
			valid_for: Option<T::BlockNumber>,
		) -> DispatchResult { 
			let owner = ensure_signed(origin)?;
			ensure!(name.len() <= 65, Error::<T>::AttributeCreationFailed);

			Self::create_attribute(&owner, &identity, &name, &value,valid_for)?;

			Self::deposit_event(Event::AttributeAdded { 
				identity,
				name,
				valid_for
			});
			
			Ok(())
		}
		//	Revokes an attribute from an identity
		//	Sets its expiration period to the actual block number 
		#[pallet::weight(0)]
		pub fn revoke_attribute(
			origin: OriginFor<T>,
			identity: T::AccountId, 
			name: Vec<u8>
		) -> DispatchResult { 
			let owner = ensure_signed(origin)?;
			ensure!(name.len() <= 64, Error::<T>::AttributeRemovalFailed);
			Self::reset_attribute(owner, &identity, &name)?;

			let now = frame_system::Pallet::<T>::block_number();
			Self::deposit_event(Event::AttributeRevoked { 
				identity,
				name,
				now
			});

			Ok(())
		}
		#[pallet::weight(0)]
		pub fn delete_attribute(
			origin: OriginFor<T>,
			identity: T::AccountId,
			name: Vec<u8>
		) -> DispatchResult { 
			let owner = ensure_signed(origin)?;
			//	Check if we are the owner 
			Self::is_owner(&identity, &owner)?;
			//	check if the name is valid 
			ensure!(name.len() <= 64, Error::<T>::AttributeRemovalFailed);
			//	Get Attribute and Hash Identifier 
			let result = Self::attribute_and_id(&identity, &name);
			//	Get the hash identifier for this attribute 
			match result {
                Some((_, id)) => <AttributeOf<T>>::remove((&identity, &id)),
                None => return Err(Error::<T>::AttributeRemovalFailed.into()),
            }
			let now = frame_system::Pallet::<T>::block_number();
			UpdatedBy::<T>::insert(&identity, (owner, &now, T::Time::now()));
			Self::deposit_event(Event::AttributeDeleted { 
				identity,
				name, 
				now

			});
			Ok(())
		}
		//	Execute off chain signed transaction 
		#[pallet::weight(0)]
		pub fn execute(
			origin: OriginFor<T>,
			transaction: AttributeTransaction<T::Signature, T::AccountId>
		) -> DispatchResult { 
			let owner = ensure_signed(origin)?;

			let mut encoded = transaction.name.encode();
			//	encode: converts the self into an owned vector 
			encoded.extend(transaction.value.encode());
			encoded.extend(transaction.validity.encode());
			encoded.extend(transaction.identity.encode());

			//	Execute 
			Self::signed_attribute(owner, &encoded, &transaction)?;
            Self::deposit_event(Event::AttributeTransactionExecuted(transaction));
			Ok(())
		}
	}
}
