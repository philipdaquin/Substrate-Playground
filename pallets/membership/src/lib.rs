#![cfg_attr(not(feature = "std"), no_std)]

// This file is part of Gamme Finance.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Functions for the Permissioned Membership pallet.

use std::marker::PhantomData;
use codec::{Encode, Decode, Codec};
use frame_support::{dispatch::{GetCallMetadata, DispatchInfo, Dispatchable, TransactionPriority}, 
	unsigned::{TransactionValidityError, TransactionValidity}, 
	pallet_prelude::{ValidTransaction, TransactionLongevity, InvalidTransaction},
	traits::{UnixTime}};
pub use pallet::*;
use sp_runtime::traits::{ DispatchInfoOf, SignedExtensionMetadata};
use sp_runtime::traits::SignedExtension;
mod types;
pub use crate::types::*;
mod functions;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::fmt::Debug;

#[frame_support::pallet]
pub mod pallet {
	
use sp_runtime::traits::{AtLeast32Bit, StaticLookup};
pub type Moment = u64;

use super::*;
use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::{ChangeMembers, InitializeMembers}, Blake2_128Concat};
	use frame_system::pallet_prelude::*;
	use frame_support::pallet_prelude::PhantomData;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// Required origin for adding a member
		type AddOrigin: EnsureOrigin<Self::Origin>;
		//	Reuqired origin for removing a member
		type RemoveOrigin: EnsureOrigin<Self::Origin>;
		//	The receiver of the signal for when the membership has changed 
		type MembershipChange: ChangeMembers<Self::AccountId>;
		// The receiver of the signal for when the membership 
		type MembershipInitialised: InitializeMembers<Self::AccountId>;
		// Type used for expressing timestamp.
		type Moment: AtLeast32Bit + Parameter + Default + Copy + From<u64>;
		//	force Orign 
		type ForceOrigin: EnsureOrigin<Self::Origin>;
		type UnixTime: UnixTime;
	
		
	
		
	
	}
	pub type AccountId<T> = <T as frame_system::Config>::AccountId;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Member<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn membership)]
	pub type Permission<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, Role),
		bool,
		ValueQuery
	>;
	#[pallet::storage]
	#[pallet::getter(fn roles)]
	pub type Roles<T> = StorageValue<_, Vec<Role>, ValueQuery>;


	#[pallet::storage]
	#[pallet::getter(fn admins)]
	pub type Admin<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, 
		bool,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> { 
		pub members: Vec<T::AccountId>,
		pub phantom: PhantomData<T>
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> { 
		fn default() -> Self { 
			Self { 
				members: Vec::new(),
				phantom: Default::default()
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> { 
		fn build(&self) { 
			use sp_std::collections::btree_set::BTreeSet;
			let btree_set: BTreeSet<_> = self.members.iter().collect();
			
			let mut members = self.members.clone();
			members.sort();
			
			T::MembershipInitialised::initialize_members(&members);
			Member::<T>::put(members);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		RolesCreated { 
			who: T::AccountId,
			pallet_name: Vec<u8>,
			permission: Vec<u8>,
			now: T::Moment,
		},
		AccessGranted { 
			who: T::AccountId,
			pallet_name: Vec<u8>,
			permission: Permissions,
			now: T::Moment, 
		},
		RevokeUserAccess { 
			who: T::AccountId, 
			pallet_name: Vec<u8>,
			now: T::Moment, 
		},
		AdminAdded { 
			who: T::AccountId
		},
		MemberAdded { 
			who: T::AccountId,
			now: T::Moment
		},
		RevokeMembership { 
			who: T::AccountId,
			pallet_name: Vec<u8>,
			permission: Permissions,
			now: T::Moment
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		MemberAlreadyExists,
		NoneExistentMember,
		RoleAlreadyExists,
		UnAuthorisedCall,
		RevokeUserAccess,
		Unknown,


	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_role(
			origin: OriginFor<T>,
			pallet_name: Vec<u8>,
			permission: Permissions,
			new_member: <T::Lookup as StaticLookup>::Source
		) -> DispatchResult {
			T::AddOrigin::ensure_origin(origin)?;

			let new_member = T::Lookup::lookup(new_member)?;
			//	Role specified for a new_user 
			let role = Role { 
				pallet_name: pallet_name.clone(), 
				permission: permission.clone(),
			};

			ensure!(!Self::do_add_role(role), Error::<T>::RoleAlreadyExists);
			//	Added a role for a Member
			Permission::<T>::insert(
				(new_member.clone(), 
				Role { 
				pallet_name: pallet_name.clone(), 
				permission: permission.clone(),
				}
			), true);

			let now: T::Moment = (T::UnixTime::now().as_secs() / 60).into();


			Self::deposit_event(Event::RolesCreated { 
				who: new_member,
				pallet_name,
				permission: permission.as_bytes().to_vec(),
				now,
			});

			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn assign_role(
			origin: OriginFor<T>,
			acc: <T::Lookup as StaticLookup>::Source, 
			role: Role,
		) -> DispatchResult { 
			let sender = ensure_signed(origin)?;
			let new = T::Lookup::lookup(acc)?;
			
			ensure!(
				!Self::verify_manage_access(role.pallet_name.clone(), new.clone()), 
				Error::<T>::UnAuthorisedCall
			);
			let now: T::Moment = (T::UnixTime::now().as_secs() / 60).into();
			//let time: T::Moment = T::Time::now().into();
			Permission::<T>::insert((new.clone(), role.clone()), true);
			Self::deposit_event(Event::AccessGranted { 
				who: new,
				pallet_name: role.pallet_name,
				permission: Permissions::Management,
				now,
			});

			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn revoke_user_access(
			origin: OriginFor<T>,
			acc: <T::Lookup as StaticLookup>::Source, 
			role: Role, 
		) -> DispatchResult { 
			let sender = ensure_signed(origin)?;
			let new = T::Lookup::lookup(acc)?;
			//	Check if the user has Management Permission 
			ensure!(
				!Self::verify_manage_access(role.pallet_name.clone(), new.clone()), 
				Error::<T>::UnAuthorisedCall
			);

			let now: T::Moment = (T::UnixTime::now().as_secs() / 60).into();
			//	Remove acc from Permissions storage 
			Permission::<T>::remove((new.clone(), role.clone()));
			Self::deposit_event(Event::RevokeUserAccess { 
				who: new, 
				pallet_name: role.pallet_name,
				now
			});


			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn add_admin(
			origin: OriginFor<T>,
			acc: <T::Lookup as StaticLookup>::Source
		) -> DispatchResult { 
			T::ForceOrigin::ensure_origin(origin)?;
			let acc = T::Lookup::lookup(acc)?;

			Admin::<T>::insert(	&acc, true );
			Self::deposit_event(Event::AdminAdded { 
				who: acc
			});


			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn add_member(
			origin: OriginFor<T>,
			new_member: <T::Lookup as StaticLookup>::Source
		) -> DispatchResult { 
			T::AddOrigin::ensure_origin(origin)?;
			let new_member = T::Lookup::lookup(new_member)?;
			let mut members = Member::<T>::get();
			let location_member = members.binary_search(&new_member)
				.err()
				.ok_or(Error::<T>::MemberAlreadyExists)?;
			members.insert(location_member, new_member.clone());

			let now: T::Moment = (T::UnixTime::now().as_secs() / 60).into();
			Member::<T>::put(&members);
			Self::deposit_event(Event::MemberAdded { 
				who: new_member,
				now
			});
			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn remove_member(
			origin: OriginFor<T>,
			curr_member: <T::Lookup as StaticLookup>::Source
		) -> DispatchResult { 
			T::RemoveOrigin::ensure_origin(origin)?;
			let curr_member = T::Lookup::lookup(curr_member)?;
			let mut members = Member::<T>::get();
			let location_member = members.binary_search(&curr_member)
				.ok()
				.ok_or(Error::<T>::NoneExistentMember)?;
			members.remove(location_member);
			Member::<T>::put(members);
			Ok(())
		}
	} 	
	
}
