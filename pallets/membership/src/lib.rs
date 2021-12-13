#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::EnsureOrigin;
use frame_system::RawOrigin;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use sp_std::if_std;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, Blake2_128Concat};
	use frame_system::pallet_prelude::*;
	

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}
	pub type AccountId<T> = <T as frame_system::Config>::AccountId;
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn company)]
	pub type Company<T> = StorageValue<_, Vec<AccountId<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T> = StorageMap<
		_,
		Blake2_128Concat,
		AccountId<T>, 
		Vec<AccountId<T>>,
		ValueQuery
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreatedCompany { 
			acc: T::AccountId, 
			company: Vec<u8>
		},
		AddedMembers { 
			acc: T::AccountId, 
			company: Vec<u8>
		}

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		CompanyDoesNotExist,
		AlreadyAMember,
		InvalidOrganisation
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_company(
			origin: OriginFor<T>,
			company_name: Vec<u8>
		) -> DispatchResult { 
			let sender = ensure_signed(origin)?;

			let mut company_info = Company::<T>::get();
			ensure!(company_info.contains(&sender), Error::<T>::CompanyDoesNotExist);
			company_info.push(sender.clone());

			Company::<T>::put(company_info);

			Did::Pallet::<T>::create_attribute(
				&sender, 
				&sender,
				b"Org",
				&company_name, 
				None
			)?;
			
			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn add_to_company(
			origin: OriginFor<T>,
			acc: AccountId<T>
		) -> DispatchResult { 
			let company = ensure_signed(origin)?;

			let company_id = Company::<T>::get();
			ensure!(company_id.contains(&company), Error::<T>::InvalidOrganisation);

			let mut members = Members::<T>::get(company);

			if !members.contains(&acc) { 
				members.push(acc.clone())
			} else { 
				return Err(Error::<T>::AlreadyAMember.into())
			}
			Did::Pallet::<T>::create_delegate(
				&company,
				&company,
				&acc,
				&b"Member".to_vec(),
				None,
			)?;
			Ok(())
		}
	}
	impl<T: Config> Pallet<T> { 
		pub fn verify_member(acc: &T::AccountId) -> bool { 
			let organisation= Company::<T>::get();
			for member in organisation.iter() { 
				if Did::Pallet::<T>::valid_delegate(member,
				&b"Member".to_vec(), &acc).is_ok() { 
					return true 
				}
			}
			return false 
		}
	}
}
//* To Learn Custom Origins  */
//* Members of Organisation can only call this function/
//	Check if the member is part of a consortium member is invoking a dispatch 
pub struct EnsureOrg<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> EnsureOrigin<T::Origin> for EnsureOrg<T> { 
	type Success = T::AccountId;
	fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
		if_std! { 
			println!("inside custom origin");
		}
		o.into().and_then(|o| match o { 
			RawOrigin::Signed(ref who) 
			if Pallet::<T>::verify_member(&who) => Ok(who.clone()),
			r => Err(T::Origin::from(r)),
		})
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> T::Origin {
		T::Origin::from(RawOrigin::Signed(Default::default()))
	}

}
