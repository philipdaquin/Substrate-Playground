use super::*;
use frame_support::{dispatch::DispatchResult, traits::Time};
//  Trait Implementation 
impl<T: Config> Identity<T::AccountId, T::BlockNumber, <<T as Config>::Time as Time>::Moment, T::Signature> 
		for Pallet<T> 
	{ 
		fn identity_owner(identity: &T::AccountId) -> T::AccountId { 
			match OwnerOf::<T>::get(identity) { 
				Some(id) => id,
				None => identity.clone()
			}
		} 

		fn is_owner(identity: &T::AccountId, actual_owner: &T::AccountId) -> DispatchResult { 
			let owner = Self::identity_owner(identity);
			match owner == *actual_owner { 
				true => Ok(()),
				false => { 
					return Err(Error::<T>::NotOwner.into())
				}
			}
		}

	}