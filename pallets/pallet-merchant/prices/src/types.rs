use super::*;


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