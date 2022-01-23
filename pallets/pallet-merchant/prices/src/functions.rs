use super::*;

impl<T: Config> Price<T> { 
	pub fn new(
		id: PriceId, 
		object: Object, 
		billing_scheme: BillingScheme,
		created_by: T::Account,
		currency: CurrencyId, 
		livemode: bool,
		product: ProductId, 
		description: Vec<u8>,
		tier_mode: Option<TiersMode<DepositBalance, Balance>>,
		puchase_type: Type, 
		unit_amount: Balance, 
		unit_amoutn_decimal: Option<Decimal>,
	) -> Self { 
		Price { 
			id: Self::get_id(),
			object, 
			created_by: Createdby::<T>::new(created_by.clone()),
			billing_scheme,
			created_at: Pallet::<T>::get_time(),
			currency,
			livemode, 
			description: Pallet::<T>::convert_string(description.clone()),
			product,
			tiers_mode,
			purchase_type,
			unit_amount,
			unit_amount_decimal,			
		}
	}
}

impl<T: Config> Pallet<T> { 
	fn ensure_root_or_signed(
		origin: T::Origin
	) -> Result<RawOrigin<T::AccountId>, DispatchError> { 
		match origin.into() { 
			Ok(frame_system::RawOrigin::Root) => Ok(
				frame_system::RawOrigin::Root
			),
			Ok(frame_system::RawOrigin::Signed(acc)) => Ok(
				frame_system::RawOrigin::Signed(acc)
			),
			_ => { 
				return Err(sp_runtime::DispatchError::BadOrigin)
			}
		}
	}
	fn get_id() -> [u8; 16] {
		let payload = (
			T::IdRandomness::random(&b"priceid"[..]).0,
			frame_system::Pallet::<T>::block_number(),
		);
		payload.using_encoded(blake2_128)
	}
	fn convert(interval: Interval) -> T::BlockNumber { 
		match interval { 
			Interval::Month => T::Month,
			Interval::Week => T::Week,
			Interval::Day => T::Day,
			Interval::Year => T::Year
		}
	}
	fn convert_string(string: Vec<u8>) -> BoundedVec<u8, T::StringLimit> { 
		let mut bounded_string: BoundedVec<u8, T::StringLimit> =
			string.clone().try_into().map_err(Error::<T>::BadMetadata);
		bounded_string
	}
	fn get_time() -> T::Moment { 
		let now_as_mins: T::Moment = (T::UnixTime::now().as_secs() / ONE_MINUTE).into();
		// Truncate seconds, keep minutes
		let now_as_secs: T::Moment = now_as_mins * ONE_MINUTE.into();
		now_as_secs
	}	
}


impl<T: Config> CreatedBy<T> { 
	pub fn new(acc: T::AccountId) -> Self { 
		CreatedBy { 
			account_id: acc.clone(),
			blocknumber: frame_system::Pallet::<T>::block_number(),
			time: Self::get_time(),
		}
	}
}