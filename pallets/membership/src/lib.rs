#![cfg_attr(not(feature = "std"), no_std)]

use std::marker::PhantomData;
use scale_info::{StaticTypeInfo, TypeInfo};
use codec::{Encode, Decode, Codec};
use frame_support::{dispatch::{GetCallMetadata, DispatchInfo, Dispatchable, TransactionPriority}, unsigned::{TransactionValidityError, TransactionValidity}, pallet_prelude::{ValidTransaction, TransactionLongevity, InvalidTransaction}};
pub use pallet::*;
use sp_runtime::traits::{SignedExtension, DispatchInfoOf, SignedExtensionMetadata};
mod types;
use crate::types::*;


#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::fmt::Debug;


#[frame_support::pallet]
pub mod pallet {
	
use sp_runtime::traits::{AtLeast32Bit, StaticLookup};
use sp_std::collections::btree_set;

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
			T::AddOrigin::ensure_signed(origin)?;

			let new_member = T::Lookup::lookup(new_member)?;
			//	Role specified for a new_user 
			let role = Role { 
				pallet_name, 
				permission,
			};

			ensure!(!Self::do_add_role(&role), Error::<T>::RoleAlreadyExists);
			//	Added a role for a Member
			Permissions::<T>::insert((new_member, role.clone(), true));

			Self::deposit_event(Event::RolesCreated { 
				who: new_member,
				pallet_name,
				permission: permission.as_bytes().to_vec(),
				now: T::Moment::now(),
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
			let acc = T::Lookup::lookup(origin)?;
			
			ensure!(
				!Self::verify_manage_access(role.pallet_name.clone(), acc), 
				Error::<T>::UnAuthorisedCall
			);

			Permissions::<T>::insert((acc.clone(), role.clone()), true);
			Self::deposit_event(Event::AccessGranted { 
				who: acc,
				pallet_name: role.pallet_name,
				permission: Permissions::Management,
				now: T::Moment::now()
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
			//	Check if the user has Management Permission 
			ensure!(
				!Self::verify_manage_access(role.pallet_name.clone(), acc), 
				Error::<T>::UnAuthorisedCall
			);
			//	Remove acc from Permissions storage 
			Permissions::<T>::remove((acc.clone(), role.clone()));
			Self::deposit_event(Event::RevokeUserAccess { 
				who: acc, 
				pallet_name: role.pallet_name,
				now: T::Moment::now()
			});


			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn add_admin(
			origin: OriginFor<T>,
			acc: <T::Lookup as StaticLookup>::Source
		) -> DispatchResult { 
			T::ForceOrigin::ensure_origin(origin)?;

			Admin::<T>::insert(
				T::Lookup::lookup(acc),
				true
			);
			Self::deposit_event(Event::AdminAdded { 
				who: acc
			});


			Ok(())
		}
	} 
	impl<T: Config> Pallet<T> { 
		fn do_add_role(
			role: Role,
		) -> bool { 
			let all_roles = Role::<T>::get();
			
			if all_roles.contains(&role) { 
				false
			}
			Role::append(role.clone());
			true 

		}
		fn verify_manage_access(
			pallet_name: Vec<u8>,
			acc: T::AccountId, 
		) -> bool { 
			let all_roles = Roles::<T>::get();
			let role = Role { 
				pallet_name,
				permission: Permissions::Management
			};
			if all_roles.contains(role.clone) && Permissions::<T>::get((acc, role.clone() )) { 
				true
			}
			false
		}
		pub fn verify_access(
			acc: T::AccountId, 
			pallet_name: Vec<u8>,
		) -> bool { 
			let role_management = Role { 
				pallet_name,
				permission: Permission::Management
			};
			let role_executor = Role { 
				pallet_name,
				permission: Permission::Executors
			};
			let all_roles = Roles::<T>::get();

			let management = Permission::<T>::get((acc, role_management)).ok_or(Error::<T>::Unknown)?;
			let executor = Permission::<T>::get((acc, role_executor)).ok_or(Error::<T>::Unknown)?;
			
			if all_roles.contains(&role_management) && management || all_roles.contains(&role_executor) && executor { 
				true
			}
			false

		}
	}
}
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct Authorize<T: Config + Send + Sync>(PhantomData<T>);

/// Debug impl for the `Authorize` struct.
impl<T: Config + Send + Sync> Debug for Authorize<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "Authorize")
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config + Send + Sync> SignedExtension for Authorize<T> where 
	T::Call: Dispatchable<Info = DispatchInfo> + GetCallMetadata,
{ 
	type AccountId = T::AccountId; 
	// The type which encodes the sender identity 
	type Call = T::Call;
	//	the type which encodes the call to be dispatched 
	type AdditionalSigned = ();
	//	any additional adata that will go into the signed payload
    type Pre = ();
	//	the type that encodes information that can be passed from pre-dispatch to post dispatch
    const IDENTIFIER: &'static str = "Authorize";
	//	The unique identifier of this signed extension 

	//	 Construct any additional data that should be in the signed payload of the transaction 
	//	This will perform anyu pre-signature verification checks and returns and error if needed
    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

	//	Validate a signed transaction for the transaction queue
	fn validate( 
		&self, 
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		_len: usize, 
	) -> TransactionValidity { 
		let md = call.get_call_metadata();
		
		//	Check Transaction Queues for Admin extrinsics, don't filter ones with key access
		if Admin::<T>::contains_key(who.clone()) { 
			print!("Access Granted!");

			Ok(ValidTransaction { 
				priority: info.weight as TransactionPriority,
				longevity: TransactionLongevity::max_value(),
				propagate: true, 
				..Default::default()
			})
		} else if Pallet::<T>::verify_access(who.clone(), md.pallet_name.as_bytes.to_vec()) { 
			print!("Access Granted");

			Ok(ValidTransaction { 
				priority: info.weight as TransactionPriority,
				longevity: TransactionLongevity::max_value(),
				propagate: true, 
				..Default::default()
			})

		} else { 
			print!("Access Denied");
			Err(InvalidTransaction::Call.into())
		}
	}
}
