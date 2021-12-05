// This file is part of Gamma Finance.

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

//! Functions for the Assets pallet.

#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
use sp_runtime::MultiSignature;
use sp_runtime::traits::IdentifyAccount;
use sp_runtime::traits::Verify;
mod builder;
use crate::builder::*;

mod types;
use crate::types::*;

//* Subscription Model based Payment System: */
//	- Connect and Role based system -> Allow users to rate the merchants 
//	- use proxy pallet to force user to pay else -> cancel subscription 	

pub use pallet::*;
use codec::{Decode, Encode, HasCompact};
use frame_support::{pallet_prelude::*, ensure, storage::child, PalletId,
	traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons, UnixTime,
		fungibles::{Inspect, Mutate, Transfer}},
	sp_runtime::{traits::{AccountIdConversion, AtLeast32BitUnsigned, Saturating, Zero, Hash, AtLeast32Bit}, ArithmeticError,
	sp_std::prelude::*
	}
};

use scale_info::TypeInfo;
//	
pub type CurrencyId = u32;
/// Balance of an account.
pub type Balance = u128;
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type PaymentIndex = u32;

#[frame_support::pallet]
pub mod pallet {
	use frame_system::{pallet_prelude::OriginFor, ensure_signed};

use super::*;
	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		
		//	The currency that this fund accepts
		type Currency: Currency<Self::AccountId>;

		//	'PalletId' for the Subscription Pallet 
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		//	Versatile Assets 
		type Assets: 
		Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
		+ Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
		+ Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
		
		// Type used for expressing timestamp.
		type Moment: AtLeast32Bit + Parameter + Default + Copy + From<u64>;

		//	The units for balance
		type Balance: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;
		
		// The origin which may forcible create or destroy a payment 
		//	The origin that executes dispatchable for user recurring payment 
		type ForceOrigin: EnsureOrigin<Self::Origin>;

		/// The maximum length of a name or symbol stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		//	The amount to be held on deposit by the depositor of a Payment Plan 
		type SubmissionDeposit: Get<BalanceOf<Self>>;
	}

	
	pub type SubscriptionIndex = u32; 
	
	pub type BalanceOf<T> = <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
	
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn paymentid)]
	pub type PaymentId<T> = StorageValue<_, PaymentIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn subscribe)]
	pub type Subscriptions<T> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, 
		(SubscriptionIndex, Subscription<AccountId, Moment>),
		ValueQuery
	>;
	#[pallet::storage]
	#[pallet::getter(fn subscription_id)]
	pub type SubscriptionId<T> = StorageValue<_, SubscriptionIndex, ValueQuery>; 

	#[pallet::storage]
	#[pallet::getter(fn info)]
	pub type PaymentInfo<T> = StorageMap<
		_,
		Blake2_128Concat,
		PaymentIndex,
		PaymentPlan<AccountId, Balance>,
		ValueQuery
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PaymentPlanCreated { 
			merchant: T::AccountId,
			id: PaymentIndex, 
			now: T::Moment,
		},
		SubcriptionCreated {
			subscriber: T::AccountId, 
			id: PaymentIndex,
			now: T::Moment
		},
		SubcriptionCancelled { 
			subcriber: T::AccountId, 
			id: PaymentIndex, 
			now: T::Moment
		},
		PaymentSent { 
			from: T::AccountId, 
			to: T::AccountId, 
			amount: BalanceOf<T>,
			id: PaymentIndex,
			now: T::Moment
		}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		PaymentPlanAlreadyExist,
	}
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_payment_system(
			origin: OriginFor<T>, 
			#[pallet::compact] required_payment: T::Balance,
			#[pallet::compact] frequency: Frequency,
			#[pallet::compact] name: Vec<u8>
		) -> DispatchResult { 
			let merchant = ensure_signed(origin)?;
			//	ensure merchants can only create payment plans
			let payment_id = Self::next_payment_id();
			
			match !Self::verify_new_plan(&payment_id) { 
				//	Does not exist, create a new one
				true => { 

					let bounded_name: BoundedVec<u8, T::StringLimit> =
					name.clone().try_into().expect("Payment Plan Name is too long");
					
					let deposit = T::SubmissionDeposit::get();

					let new_payment = Self::new_payment_plan()
						.identified_by(payment_id.clone())
						.owned_by(merchant.clone())
						.with_name(bounded_name.clone())
						.min_payment(required_payment)
						.new_deposit(Zero::zero())
						.payment_frequency(frequency)
						.build();

					let imbalance = T::Currency::withdraw(
						&merchant, 
						deposit, 
						WithdrawReasons::TRANSFER,
						ExistenceRequirement::AllowDeath
					)?;




					//	Insert to storage map 
					Self::deposit_event(Event::PaymentPlanCreated { 
						merchant,
						id: payment_id, 
						now: T::Moment::now(),
					})
				},
				_ => { 
					// ALready exists
					return Err(Error::<T>::PaymentPlanAlreadyExist);
					//	Print log
				}
			}

			Ok(())
		}
	} 
	impl<T: Config> Pallet<T> { 
		//	Does the id already exist in out storage? 
		fn verify_new_plan(id: &PaymentIndex) -> bool { 
			PaymentInfo::<T>::contains_key(id)
		}
		//	Create a new payment plan using PaymentPlanBuilder
		fn new_payment_plan() -> PaymentPlanBuilder<T::AccountId, T::Balance> { 
			PaymentPlanBuilder::<T::AccountId, T::Balance>::default()
		}
		//	Create subscription to a Payment Plan 
		fn new_subscription() -> SubscriptionBuilder<T::AccountId, T::Moment> { 
			SubscriptionBuilder::<T::AccountId, T::Moment>::default()
		}
		//	This is where recurring payments are paid into 
		pub fn fund_account_id(idx: PaymentIndex) -> T::AccountId { 
			T::PalletId::get().into_sub_account(idx)
		}
		//	Track Payment Index
		fn next_payment_id() -> Result<u32, DispatchError> {
			PaymentId::<T>::try_mutate(|index| -> Result<u32, DispatchError> {
				let current_id = *index;
				*index = index.checked_add(1).ok_or(ArithmeticError::Overflow)?;
				Ok(current_id)
			})
		}
		// 	Function to find the id associated with the fund id (child trie)
		//	Each fund stores information about it ***contributors and their ***contributions in a child trie 
		
		//	This helper function calculates the id of the associate child trie 
		pub fn id_from_index(
			index: PaymentIndex
		) -> child::ChildInfo { 
			let mut buf = Vec::new();
			buf.extend_from_slice(b"payment");
			buf.extend_from_slice(&index.to_le_bytes()[..]);

			child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
		}
		//	Put Payment under a key: user account 
		pub fn contribution_put(
			index: PaymentIndex, 
			who: &T::AccountId, 
			balance: &T::Balance
		) {
			let id = Self::id_from_index(index);
			who.using_encoded(|b| child::put(&id, b, &balance));
		}
		//	Get the value paid by the user 
		pub fn contribution_get(
			index: PaymentIndex, 
			who: &T::AccountId
		) -> BalanceOf<T> {
			let id = Self::id_from_index(index);
			who.using_encoded(|b| child::get_or_default::<BalanceOf<T>>(&id, b))
		}
		pub fn crowdfund_kill(index: PaymentIndex) {
			let id = Self::id_from_index(index);
			// The None here means we aren't setting a limit to how many keys to delete.
			// Limiting can be useful, but is beyond the scope of this recipe. For more info, see
			// https://crates.parity.io/frame_support/storage/child/fn.kill_storage.html
			child::kill_storage(&id, None);
		}
	}
}