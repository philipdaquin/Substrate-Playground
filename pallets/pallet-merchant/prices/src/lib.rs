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
	pub type ProductList<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ProductId,
		Vec<ProductInfo<T::UnixTime, BoundedVec<u8, T::StringLimit>>>,
		ValueQuery,
	>;
	// Store Product Ids owned by an organisation
	#[pallet::storage]
	#[pallet::getter(fn organisation_products)]
	pub type OrganisationProduct<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<ProductId>, ValueQuery>;

	// Quickly Access Owners of Product Id 
	#[pallet::storage]
	#[pallet::getter(fn owners)]
	pub type OwnerOf<T: Config> = StorageMap<_, Blake2_128Concat, ProductId, Option<T::AccountId>, ValueQuery>;

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
			updated_at: T::UnixTime,
			active: bool,
			name: Vec<u8>,
			description: Vec<u8>,
			livemode: bool,
			shippable: bool,
			unit_label: u32,
			product_url: Vec<u8>,
		},
		RetrieveProduct { 
			id: ProductId,
			object:  ObjectType, 
			active: bool,
			created_at: T::UnixTime,
			description: Vec<u8>,
			ipfs_cid: CID,
			livemode: bool,
			product_name: Vec<u8>,
			package_dimensions: Option<Dimensions>,
			shippable: bool,
			unit_label: u32,
			updated_at: T::UnixTime,
			url: Vec<u8>,
		},
		ListAllProducts { 
			id: ProductId,
			object:  ObjectType, 
			active: bool,
			created_at: T::UnixTime,
			description: Vec<u8>,
			ipfs_cid: CID,
			livemode: bool,
			product_name: Vec<u8>,
			package_dimensions: Option<Dimensions>,
			shippable: bool,
			unit_label: u32,
			updated_at: T::UnixTime,
			url: Vec<u8>,
		}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		BadMetadata,
		ProductDoesNotExist,
		NoOwnerFound,
		ProductNotFoundInOrg,
		Unknown
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		///	Create a new product object
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
			
			Self::do_create(
				merchant,
				active,
				name,
				description,
				package_dimensions,
				livemode,
				ipfs_cid,
				shippable,
				unit_label,
				product_url,
			)
		}

		//	Use Runtime RPC to ORG Details

		//	Retrieves the details of an existing product
		// Supply the product Id from either a product list of an owner, or the merchants
		#[pallet::weight(10)]
		pub fn retrieve_product(
			origin: OriginFor<T>,
			product_id: ProductId,
		) -> DispatchResult { 
			let merchant = T::Merchant::ensure_origin(origin)?;
			Self::verify_product_owner(merchant, product_id)?;
			
		
			let ProductInfo::<T::UnixTime, BoundedVec<u8, T::StringLimit>> {
				id,
				object, 
				active,
				created_at,
				description,
				ipfs_cid,
				livemode,
				product_name,
				package_dimensions,
				shippable,
				unit_label,
				updated_at,
				url,
			} = ProductList::<T>::get(&product_id);

			Self::deposit_event(Event::RetrieveProduct { 
				id,
				object, 
				active,
				created_at,
				description: description.to_vec(),
				ipfs_cid,
				livemode,
				product_name: product_name.to_vec(),
				package_dimensions,
				shippable,
				unit_label,
				updated_at,
				url: url.to_vec(),
			});
			Ok(())
		}

		///	Updates the specific product by setting the values of the parameters
		/// passed. Any parameters not provided will be left unchanged
		#[pallet::weight(10)]
		pub fn update_product(
			origin: OriginFor<T>,
			product_id: ProductId,
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

			//	Check organisation and product owners 
			Self::verify_product_owner(merchant, product_id)?;
			//	Find the product in the product list 
			ProductList::<T>::mutate(&product_id, |info| -> DispatchResult { 
				let details = info.as_mut().ok_or(Error::<T>::Unknown)?;

				let bounded_name: BoundedVec<u8, T::StringLimit> =
					name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
			
				let bounded_url: BoundedVec<u8, T::StringLimit> =
					product_url.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
			
				let bounded_description: BoundedVec<u8, T::StringLimit> =
					description.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
				let time_now: T::UnixTime = T::UnixTime::now();
				
				details.active = active;
				details.name = bounded_name.clone();
				details.description = bounded_description.clone();
				details.package_dimensions = package_dimensions;
				details.livemode = livemode;
				details.ipfs_cid = ipfs_cid;
				details.shippable = shippable;
				details.unit_label = unit_label;
				details.product_url = bounded_url.clone();
				details.updated_at = time_now.clone();

				Self::deposit_event(Event::ProductUpdated {
					product_id,
					updated_at: time_now,
					active,
					name,
					description,
					livemode,
					shippable,
					unit_label,
					product_url,
				});

				Ok(())
			})
		}
		///	Returns a list of your products. The products are returned
		/// sorted by creation date, with the most recently created products appearing first
		/// 
		/// Only return products that are active or inactive
		#[pallet::weight(10)]
		pub fn list_all_products(
			origin: OriginFor<T>
		) -> DispatchResult { 
			let merchant = T::Merchant::ensure_origin(origin)?;
			let org_list = OrganisationProduct::<T>::get(&merchant);
			
			//	Iterate through ProductIds and get each ProductInfo
			for products in org_list { 
				let ProductInfo::<T::UnixTime, BoundedVec<u8, T::StringLimit>> {
					id,
					object, 
					active,
					created_at,
					description,
					ipfs_cid,
					livemode,
					product_name,
					package_dimensions,
					shippable,
					unit_label,
					updated_at,
					url,
				} = ProductList::<T>::get(products);
				
				//	Emit event for each
				Self::deposit_event(Event::ListAllProducts { 
					id,
					object, 
					active,
					created_at,
					description,
					ipfs_cid,
					livemode,
					product_name,
					package_dimensions,
					shippable,
					unit_label,
					updated_at,
					url,
				});				
			}

			Ok(())
		}
		#[pallet::weight(10)]
		pub fn delete_product(
			origin: OriginFor<T>
		) -> DispatchResult { 
			let merchant = T::Merchant::ensure_origin(origin)?;

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
		fn product_owner(product_id: ProductId) -> T::AccountId { 
			//	Check if the Product List contains the id
			ensure!(!ProductList::<T>::contains_key(&product_id), Error::<T>::ProductDoesNotExist);
			//	Check if there are owners for this id
			ensure!(!OwnerOf::<T>::contains_key(&product_id), Error::<T>::NoOwnerFound);
			
			match OwnerOf::<T>::get(product_id) { 
				Some(id) => id,
				None => { return Err(Error::<T>::NoOwnerFound.into())}
			}
		}
		fn verify_product_owner(merchant: T::AccountId, product_id: ProductId) -> DispatchResult {
			let owner = Self::product_owner(product_id);

			match owner == merchant { 
				true => {
					//	check if the merchants org owns the product
					let org_products = OrganisationProduct::<T>::get(&merchant);
					ensure!(!org_products.contains(&product_id), Error::<T>::ProductNotFoundInOrg);
					
					Ok(())
				},
				false => { return Err(Error::<T>::NoOwnerFound.into())}
			}
		}
		fn store_product(
			product_id: ProductId, 
			product_info: ProductInfo<T::UnixTime, BoundedVec<u8, T::StringLimit>>, 
			merchant: T::AccountId,
		) -> DispatchResult { 
			


			ProductList::<T>::insert(&product_id, product_info);
			OrganisationProduct::<T>::append(&merchant, &product_id);
			OwnerOf::<T>::insert(&product_id, &merchant);
			
			Ok(())
		}
		fn do_create(
			merchant: T::AccountId,
			active: bool,
			name: Vec<u8>,
			description: Vec<u8>,
			package_dimensions: Option<Dimensions>,
			livemode: bool,
			ipfs_cid: CID,
			shippable: bool,
			unit_label: u32,
			product_url: Vec<u8>,
		) -> DispatchResult { 
			let id = Self::get_id();
			let now = T::UnixTime::now();
			
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
			//	Write to Db
			Self::store_product(&id, product_info.clone(), merchant);

			Self::deposit_event(Event::ProductCreated {
				product_id: id,
				object_type: ObjectType::Product,
				created_at: now, 
				livemode,
			});
			
			Ok(())	
		}
	}
}
