use std::default::{self};

use super::*;
use codec::{Encode, Decode};
use scale_info::TypeInfo;

#[derive( Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum ObjectType {
	Product,
	Person,
	Payout,
	Order, 
	PaymentIntent,
	Invoice,
	Subscription,
	Event,
}
#[derive(Encode, Decode, Default, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ProductInfo<Moment, BoundedString> { 
	pub id: ProductId,
	pub object:  ObjectType, 
	pub active: bool,
	#[codec(compact)]
	pub created_at: Moment,
	pub description: BoundedString,
	pub ipfs_cid: CID,
	pub livemode: bool,
	pub product_name: BoundedString,
	pub package_dimensions: Option<Dimensions>,
	pub shippable: bool,
	pub unit_label: BoundedString,
	pub updated_at: Moment,
	pub url: BoundedString,
}

impl Default for ObjectType { 
	fn default() -> Self {
		ObjectType::Product
	}
}
#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum TypeLabel { 
	USDT,
	Custom { 
		name: Vec<u8>,
		symbol: Vec<u8>
	}
}

impl Default for TypeLabel { 
	fn default() -> Self {
		TypeLabel::USDT
	}
}



// #[derive(Encode, Decode, Default, PartialEq, Eq, RuntimeDebug, TypeInfo)]
// pub struct Label { 
// 	name: Vec<u8>,
// 	symbol: Vec<u8>,
// }

// impl UnitLabel { 
// 	fn new_label(name: Vec<u8>, symbol: Vec<u8>) -> Label { 
// 		let bounded_name: BoundedVec<u8, T::StringLimit> =
// 		name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

// 		let bounded_symbol: BoundedVec<u8, T::StringLimit> =
// 		symbol.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
		
// 		Label::<BoundedString> { 
// 			name: bounded_name.to_vec(),
// 			symbol: bounded_symbol.to_vec(),
// 		}
// 	}
// }