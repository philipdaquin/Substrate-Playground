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
	pub created_at: Moment,
	pub description: BoundedString,
	pub ipfs_cid: CID,
	pub livemode: bool,
	pub product_name: BoundedString,
	pub package_dimensions: Option<Dimensions>,
	pub shippable: bool,
	pub unit_label: u32,
	pub updated_at: Moment,
	pub url: BoundedString,
}

impl Default for ObjectType { 
	fn default() -> Self {
		ObjectType::Product
	}
}