#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::{Randomness, UnixTime}};
use frame_system::{pallet_prelude::*, RawOrigin};
use orml_currencies::Currency;


mod types;
use crate::types::*;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;
// #[cfg(feature = "runtime-benchmarks")]

pub mod functions;
#[frame_support::pallet]
pub mod pallet {
	use sp_io::hashing::blake2_128;
	use super::*;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// Used to represent Currency Id and Balances 
		type Currency: MultiReservableCurrency<Self::AccountId>;
		// Used to generate Price Id
		type IdRandomness: Randomness<Self::Hash, Self::BlockNumber>;
		// Used to limit Strings
		type StringLimit: Get<u32>;
		// Origin from which the Prices are set at
		type Merchant: EnsureOrigin<Self::Origin>;
		//	UnixTime
		type UnixTime: UnixTime;
		// The basic amount of funds that must be reserved for the transaction
		type FlatFee: Get<DepositBalanceOf<Self>>;
		// Recurring Intervals:
		//	As Specified on Runtime, Intervals are measured as blocks
		type Month: Get<Self::BlockNumber>;
		//	YEARS: BlockNumber = MONTHS * 12;
		type Year: Get<Self::BlockNumber>;
		//	 WEEKS: BlockNumber = DAYS * 7;
		type Week: Get<Self::BlockNumber>;
		//	DAYS: BlockNumber = HOURS * 24;
		type Day: Get<Self::BlockNumber>;
	}
	//	Identifier
	pub type PriceId = [u8; 16];
	// Used to represent Prices 
	pub type BalanceOf<T> = <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	// used to represent Currency Ids
	pub type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
	//	Deposit Balance for Graduate/ Volume pricing models 
	pub type DepositBalanceOf<T> =
	<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// PriceId -> ProductId
	#[pallet::storage]
	#[pallet::getter(fn price_to)]
	pub type PricedFor<T: Config> = StorageMap<
		_, 
		Blake2_128Concat,
		PriceId, 
		ProductId, 
		ValueQuery,
	>;
	//	Account Id -> Option<PriceId>
	#[pallet::storage]
	#[pallet::getter(fn organisation_price)]
	pub type OrganisationPrice<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, 
		Option<PriceId>,
		ValueQuery,
	>;
	//	PriceId -> Price Struct
	#[pallet::storage]
	#[pallet::getter(fn price_to_id)]
	pub type PriceList<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		PriceId,
		Price<Moment, BalanceOf, CurrencyId>,
		ValueQuery,
	>;
	// Storage Helper
	#[pallet::storage]
	#[pallet::getter(fn owner_of_prices)]
	pub type PriceIdOwner<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		PriceId, 
		Option<T::AccountId>,
		ValueQuery
	>;

 
	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		PriceCreated { 
			price_id: PriceId, 
			object: Object, 
			active: bool,
			created_by: T::AccountId,
			billing_scheme: BillingScheme,
			created_at: Moment,
			livemode: bool,
			product_id: ProductId, 
			tiers_mode: Option<TiersMode<DepositBalance, Balance>>,
			currency: CurrencyId<T>,
			purchase_type: Type,
			unit_amount: BalanceOf<T>, 
			unit_amount_decimal: BalanceOf<T>,			
			
		},
		PriceDeleted { 

		},
		PriceUpdated { 

		},

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}


	//	Need to be revised to the nature of enums of rust 
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10000)]
		pub fn create_price(
			origin: OriginFor<T>,
			currency: CurrencyIdOf<T>,
			active: bool,
			livemode: bool,
			product: ProductId,
			description: Vec<u8>,
			tier_mode: Option<Tiersmode<DepositBalance, Balance>>,
			purchase_type: Type, 
			unit_amount: Balance,
			unit_amount_decimal: Option<Decimal>,
		) -> DispatchResult { 
			let sender = T::Merchant::ensure_origin(origin);
			//	Check if the product id is owned by the organisation
			products::<T>::verify_product_owner(sender, product);

			if let 



			let price_id = Self::get_id();
			let new_price = Price::<T>::new(
				price_id,
				object, 
				billing_scheme, 
				active, 
				created_by, 
				currency, 
				livemode, 
				product, 
				description, 
				tier_mode, 
				puchase_type, 
				unit_amount, 
				unit_amount_decimal
			);
			OrganisationPrice::<T>::insert(sender, price_id);
			PriceList::<T>::insert(price_id, new_price);
			PricedFor::<T>::insert(price_id, product);
			PriceIdOwner::<T>::insert(price_id, sender);

			Self::deposit_event(PriceCreated {
				price_id, 
				object, 
				active,
				created_by,
				billing_scheme,
				created_at,
				livemode,
				product_id, 
				tiers_mode,
				currency,
				purchase_type,
				unit_amount, 
				unit_amount_decimal,		
			});

			Ok(()) 
		}




	}
	
}
