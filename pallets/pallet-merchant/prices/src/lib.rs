#![cfg_attr(not(feature = "std"), no_std)]


/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::{UnixTime, Randomness}};
use frame_system::pallet_prelude::*;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

mod types;
use crate::types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
use sp_io::hashing::blake2_128;

pub type Dimensions = BTreeMap<Vec<u8>, Vec<u8>>;

pub type CID = Vec<u8>;
pub type ProductId = [u8; 16];

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		
		type IdRandomness: Randomness<Self::Hash, Self::BlockNumber>; 
		
		type UnixTime: UnixTime;
		
		type Merchant: EnsureOrigin<Self::Origin>;
		
		type StringLimit: Get<u32>;
	}
	
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	//	Product List 
	#[pallet::storage]
	#[pallet::getter(fn product_by_id)]
	pub type ProductList<T> = StorageMap<
		_,
		Blake2_128Concat,
		ProductId,
		Vec<ProductInfo<T::UnixTime, BoundedVec<u8, T::StringLimit>>>,
		ValueQuery,
	>;
	// Store Product Ids owned by an organisation
	#[pallet::storage]
	#[pallet::getter(fn organisation_products)]
	pub type OrganisationProduct<T> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<ProductId>, ValueQuery>;

	// Quickly Access Owners of Product Id 
	#[pallet::storage]
	#[pallet::getter(fn owners)]
	pub type OwnerOf<T> = StorageMap<_, Blake2_128Concat, ProductId, Option<T::AccountId>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config>  {
		// Occurs whenever a Product is created
		ProductCreated {
			product_id: ProductId,
			object_type: ObjectType,
			created_at: T::UnixTime,
			livemode: bool,
		},
		// Occurs whenever a Product is deleted 
		ProductDeleted {
			product_id: ProductId,
			object_type: ObjectType,	
			created_at: T::UnixTime,
			live: bool,
		},
		// Occurs whenever a Product is updated
		ProductUpdated {
			product_id: ProductId,
			object_type: ObjectType,
			created_at: T::UnixTime,
			live: bool,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		BadMetadata,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10)]
		pub fn create_product(
			origin: OriginFor<T>,
			active: bool,
			name: Vec<u8>,
			description: Vec<u8>,
			package_dimensions: Option<Dimensions>,
			livemode: bool,
			ipfs_cid: CID,
			shippable: bool,
			unit_label: u32,
			product_url: Vec<u8>	
		) -> DispatchResult { 
			let merchant = T::Merchant::ensure_origin(origin)?;
			
			let id = Self::get_id();
			let now = T::Time::now();
			
			let bounded_name: BoundedVec<u8, T::StringLimit> =
				name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
			
			let bounded_url: BoundedVec<u8, T::StringLimit> =
				product_url.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
			
			let bounded_description: BoundedVec<u8, T::StringLimit> =
				description.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
			
			let product_info = ProductInfo {
					 id,
					 object: ObjectType::Product, 
					 active,
					 created_at: now,
					 description: bounded_description,
					 ipfs_cid,
					 livemode,
					 product_name: bounded_name,
					 package_dimensions, 
					 shippable,
					 unit_label,
					 updated_at: now,
					 url: bounded_url,
			};
			ProductList::<T>::insert(&id, product_info);
			OrganisationProduct::<T>::append(&merchant, &id);
			OwnerOf::<T>::insert(&id, &merchant);				
			
			Self::deposit_event(Event::ProductCreated {
				product_id: id,
				object_type: ObjectType::Product,
				created_at: now, 
				livemode,
			});
			Ok(())
		}
		#[pallet::weight(10)]
		pub fn retrieve_product(
			origin: OriginFor<T>,
		) -> DispatchResult { 

			Ok(())
		}
		#[pallet::weight(10)]
		pub fn update_product(
			origin: OriginFor<T>
		) -> DispatchResult { 

			Ok(())
		}
		#[pallet::weight(10)]
		pub fn list_all_products(
			origin: OriginFor<T>
		) -> DispatchResult { 

			Ok(())
		}
		#[pallet::weight(10)]
		pub fn delete_product(
			origin: OriginFor<T>
		) -> DispatchResult { 

			Ok(())
		}
		
		
	}
	impl<T: Config> Pallet<T> { 
		fn get_id() -> [u8; 16] {
			let payload = (
				T::IdRandomness::random(&b"productId"[..]).0,
				frame_system::Pallet::<T>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}
	}
}
