
use crate::{traits::Identifier, types::AttributeId};

use super::*;
use crate::types::*;
use frame_support::{dispatch::DispatchResult, traits::Time};
use sp_core::{sr25519::Signature, blake2_256};
use sp_runtime::{ArithmeticError, traits::{Verify, CheckedAdd}};

//  Trait Implementation 
impl<T: Config> Identifier<T::AccountId, T::BlockNumber, <<T as Config>::Time as Time>::Moment, T::Signature>
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
        //  Validates if a delegate belongs to an identity and it has not expired 
        fn valid_delegate(
            identity: &T::AccountId,
            delegate_type: &[u8],
            delegate: &T::AccountId
        ) -> DispatchResult { 
            ensure!(delegate_type.len() <= 64, Error::<T>::InvalidDelegate);
            ensure!(
                Self::valid_listed_delegate(identity, delegate_type, delegate).is_ok() ||
                Self::is_owner(identity, delegate).is_ok(), Error::<T>::InvalidDelegate
            );
            Ok(())
        }
        fn valid_listed_delegate(
            identity: &T::AccountId,
            delegate_type: &[u8],
            delegate: &T::AccountId,
        ) -> DispatchResult { 
            //  access delegate of 
            ensure!(DelegateOf::<T>::contains_key((identity, delegate_type, delegate)), Error::<T>::InvalidDelegate);
            let validity = DelegateOf::<T>::get((identity, delegate_type, delegate));
            
            //  Check if Delegates are within expiration 
            //  Expiration time is application specific and dependent on the security requirements of the identity owner 
            let now = frame_system::Pallet::<T>::block_number();

            match validity.unwrap() > now {  
                true => Ok(()),
                _ => { 
                    return Err(Error::<T>::InvalidDelegate.into())
                }
            }
        }
        fn create_delegate(
            owner: &T::AccountId,
            identity: &T::AccountId,
            delegate: &T::AccountId,
            delegate_type: &[u8],
            valid_for: Option<T::BlockNumber>,
        ) -> DispatchResult {

            ensure!(
                Self::is_owner(identity, delegate).is_ok() || 
                !Self::valid_listed_delegate(identity, delegate_type, delegate).is_ok() || 
                owner != delegate, Error::<T>::InvalidDelegate
            );
            let now = frame_system::Pallet::<T>::block_number();
            let validity: T::BlockNumber = match valid_for {
                Some(blocks) => now + blocks,
                None => u32::max_value().into(),
            };
            DelegateOf::<T>::insert(
                (&identity, delegate_type, delegate), 
                Some(validity)
            );
            Ok(())

        }
    
        fn check_signature(
            signature: &<T as pallet::Config>::Signature, 
            msg: &[u8], 
            signer: &T::AccountId
        ) -> DispatchResult { 
            //  Signature: Verify means of signature verification 
            if signature.verify(msg, signer) {
                Ok(())
            } else {
                Err(Error::<T>::BadSignature.into())
            }
         
        }
        
        fn create_attribute(
            owner: &AccountId<T>,
            identity: &AccountId<T>, 
            name: &[u8],
            value: &[u8],
            valid_for: Option<T::BlockNumber>
        ) -> DispatchResult { 
            Self::is_owner(identity, owner)?;

            //   Check for duplicates
            ensure!(Self::attribute_and_id(identity, name).is_some(), Error::<T>::AttributeCreationFailed);

            let time_now: Moment<T> = T::Time::now();
            let now = frame_system::Pallet::<T>::block_number();
            
            let validity: T::BlockNumber = match valid_for { 
                Some(block) => now + block,
                None => u32::max_value().into()
            };
            let nonce = Self::get_next_nonce(identity, name)?;
            let id = (&identity, name, nonce).using_encoded(blake2_256);
            
            AttributeOf::<T>::insert(
                (owner, id), 
                Attribute::<T::BlockNumber, Moment<T>> { 
                    name: (name).to_vec(),
                    value: (value).to_vec(),
                    validity,
                    creation: time_now,
                    nonce
            });
            UpdatedBy::<T>::insert(
                identity,
                (owner, now, time_now)
            );
            
            Ok(())
        }
        
        //  Updates the attribute validity to make it expire and invalid 
        fn reset_attributes(
            owner: T::AccountId, 
            identity: &T::AccountId,
            name: &[u8]
        ) -> DispatchResult {
            Self::is_owner(identity, &owner)?;
            let result = Self::attribute_and_id(identity, name);
            let now = frame_system::Pallet::<T>::block_number();
            match result {
                Some((mut attribute, id)) => {
                    attribute.validity = <frame_system::Pallet<T>>::block_number();
                    <AttributeOf<T>>::mutate((&identity, id), |a| *a = attribute);
                }
                None => return Err(Error::<T>::AttributeResetFailed.into()),
            }
            UpdatedBy::<T>::insert(
                identity,
                (owner, now, T::Time::now())
            );
            
            Ok(())
        }
        //  Validatres if an attribute belongs to an identity and it has not expired 
        fn valid_attribute(
            identity: &T::AccountId, 
            name: &[u8],
            value: &[u8]
        ) -> DispatchResult { 
            ensure!(name.len() <= 64, Error::<T>::InvalidAttribute);
            let result = Self::attribute_and_id(identity, name);
            
            let (attribute, _) = match result { 
                Some((attribute, id)) => (attribute, id),
                None => { 
                    return Err(Error::<T>::InvalidAttribute.into())
                }
            };
            let now = frame_system::Pallet::<T>::block_number();
            if (attribute.validity > now) && (attribute.value == value.to_vec()) { 
                Ok(())
            } else { 
                return Err(Error::<T>::InvalidAttribute.into())
            }
        }   
        
        //  Returns the attribute and its hash identifier  
        //  Uses a nonce to keep track of identifier making them unique after attribute deletion
        fn attribute_and_id(
            identity: &T::AccountId,
            name: &[u8],
        ) ->Option<AttributeId<T::BlockNumber, <<T as Config>::Time as Time>::Moment>> { 
            let nonce = AttributeNonce::<T>::get((identity, name));

            let lookup_nonce = match nonce { 
                0u64 => 0u64,
                _ => nonce - 1u64
            };
            let id = (&identity, name, lookup_nonce).using_encoded(blake2_256);
            if AttributeOf::<T>::contains_key((&identity, &id)) { 
                Some((Self::attribute_of((identity, id)), id))
            } else { 
                None
            }
        }
        fn valid_signer(
            identity: &T::AccountId,
            signature: &<T as pallet::Config>::Signature,
            msg: &[u8],
            signer: &T::AccountId,
        ) -> DispatchResult { 
            Self::valid_delegate(
                &identity,
                b"x25519VerificationKey2018",
                &signer
            )?;
            Self::check_signature(&signature, &msg, &signer)?;
            Ok(())
        }

	}
impl<T: Config> Pallet<T> { 
    fn get_next_nonce(identity: &T::AccountId, name: &[u8]) -> Result<u64, DispatchError> { 
        AttributeNonce::<T>::mutate((identity, name.to_vec()), |nonce| -> Result<u64, DispatchError> { 
            let curr = *nonce;
            *nonce = nonce.checked_add(1).ok_or(ArithmeticError::Overflow)?;
            Ok(*nonce)
        })
    }
   pub(super) fn signed_attribute(
        who: T::AccountId, 
        encoded: &[u8],
        transaction: &AttributeTransaction<T::Signature, T::AccountId>
    ) -> DispatchResult { 
        //  Verify that the data was signed by the owner or a not expired signer delegate
        Self::valid_signer(
            &transaction.identity,
            &transaction.signature,
            encoded,
            &transaction.signer
        )?;
        //  Check attribute is owned by signer
        Self::is_owner(&transaction.identity, &transaction.signer)?;
        let now = frame_system::Pallet::<T>::block_number();
        ensure!(transaction.name.len() <= 64, Error::<T>::BadTransaction);

        let valid = now + transaction.validity.into() ;
        if valid > now { 
            Self::create_attribute(
                &who,
                &transaction.identity,
                &transaction.name,
                &transaction.value,
                Some(transaction.validity.into())
            );
        } else { 
            Self::reset_attributes(who, &transaction.identity, &transaction.name)?;
        }
        Ok(())

    }   
}