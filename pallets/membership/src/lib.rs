#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
mod types;
use crate::types::*;


#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};



#[frame_support::pallet]
pub mod pallet {
	
use sp_std::collections::btree_set;

use super::*;
use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::{ChangeMembers, InitializeMembers}};
	use frame_system::pallet_prelude::*;

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
	}
	pub type AccountId<T> = <T as frame_system::Config>::AccountId;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T> = StorageValue<_, Vec<AccountId<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn membership)]
	pub type Membership<T> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, Permissions),
		bool,
		ValueQuery
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
			Members::<T>::put(members);
		}
	}


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {



	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {



	} 
	impl<T: Config> Pallet<T> { 

	}
}
