// #![cfg_attr(not(feature = "std"), no_std)]

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
// use scale_info::{StaticTypeInfo, TypeInfo};

// // Functions for the Permissioned Membership pallet.
// use sp_runtime::traits::SignedExtension;
// use frame_support::{ensure, traits::Get};
// use sp_runtime::{DispatchError, DispatchResult};

// use super::*;
// use crate::types::*;





// impl<T: Config> Pallet<T> { 
//     pub(crate) fn do_add_role(
//         role: Role,
//     ) -> bool { 
//         let all_roles = Role::<T>::get();
        
//         if all_roles.contains(&role) { 
//            return  false
//         }
//         Role::<T>::append(role.clone());
//         true 

//     }
//     pub(super) fn verify_manage_access(
//         pallet_name: Vec<u8>,
//         acc: T::AccountId, 
//     ) -> bool { 
//         let all_roles = Role::<T>::get();
//         let role = Role { 
//             pallet_name,
//             permission: Permission::Management
//         };
//         if all_roles.contains(&role) && Permission::<T>::get((acc, role.clone() )) { 
//             return true
//         }
//         false
//     }
//     pub fn verify_access(
//         acc: T::AccountId, 
//         pallet_name: Vec<u8>,
//     ) -> bool { 
//         let role_management = Role { 
//             pallet_name,
//             permission: Permission::Management
//         };
//         let role_executor = Role { 
//             pallet_name,
//             permission: Permission::Executors
//         };
//         let all_roles = Role::<T>::get();

//         let management = Permission::<T>::get((acc, role_management));
//         let executor = Permission::<T>::get((acc, role_executor));
        
//         if all_roles.contains(&role_management) && management || all_roles.contains(&role_executor) && executor { 
//             return true
//         }
//         return  false
//     }
// }

// // Signed Transactions 
// #[derive(Encode, Decode, Clone, Eq, PartialEq, scale_info::TypeInfo)]
// pub struct Authorize<T: Config + Send + Sync>(PhantomData<T>);

// /// Debug impl for the `Authorize` struct.
// impl<T: Config + Send + Sync> Debug for Authorize<T> {
//     #[cfg(feature = "std")]
//     fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
//         write!(f, "Authorize")
//     }

//     #[cfg(not(feature = "std"))]
//     fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
//         Ok(())
//     }
// }


// impl<T: Config + Send + Sync> SignedExtension for Authorize<T> where 
// 	T::Call: Dispatchable<Info = DispatchInfo> + GetCallMetadata, T: scale_info::TypeInfo
// { 
// 	type AccountId = T::AccountId; 
// 	// The type which encodes the sender identity 
// 	type Call = T::Call;
// 	//	the type which encodes the call to be dispatched 
// 	type AdditionalSigned = ();
// 	//	any additional adata that will go into the signed payload
//     type Pre = ();
// 	//	the type that encodes information that can be passed from pre-dispatch to post dispatch
//     const IDENTIFIER: &'static str = "Authorize";
// 	//	The unique identifier of this signed extension 

// 	//	 Construct any additional data that should be in the signed payload of the transaction 
// 	//	This will perform anyu pre-signature verification checks and returns and error if needed
//     fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
//         Ok(())
//     }

// 	//	Validate a signed transaction for the transaction queue
// 	fn validate( 
// 		&self, 
// 		who: &Self::AccountId,
// 		call: &Self::Call,
// 		info: &DispatchInfoOf<Self::Call>,
// 		_len: usize, 
// 	) -> TransactionValidity { 
// 		let md = call.get_call_metadata();
		
// 		//	Check Transaction Queues for Admin extrinsics, don't filter ones with key access
// 		//	Check if Signed Transactions ahve Admin Keys 
// 		if Admin::<T>::contains_key(who.clone()) { 
// 			print!("Access Granted!");

// 			Ok(ValidTransaction { 
// 				priority: info.weight as TransactionPriority,
// 				longevity: TransactionLongevity::max_value(),
// 				propagate: true, 
// 				..Default::default()
// 			})
// 		//	Check if Signed Transactions have Verified Access 
// 		} else if Pallet::<T>::verify_access(who.clone(), md.pallet_name.as_bytes().to_vec()) { 
// 			print!("Access Granted");

// 			Ok(ValidTransaction { 
// 				priority: info.weight as TransactionPriority,
// 				longevity: TransactionLongevity::max_value(),
// 				propagate: true, 
// 				..Default::default()
// 			})

// 		} else { 
// 			print!("Access Denied");
// 			Err(InvalidTransaction::Call.into())
// 		}
// 	}
// }
