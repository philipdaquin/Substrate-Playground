use codec::{Encode, Decode};
use frame_support::RuntimeDebug;

use super::*;
//  Executors can trigger dispatch functions
#[derive(Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Membership { 
    Management,
    Executors, 
}
//  Executors are your Buyers and Sellers
#[derive(Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Executors { 
    Seller,
    Buyer, 
}

impl Membership { 
    fn as_bytes(&self) -> &[u8] { 
        match self { 
            Membership::Management => b"Membership::Management",
            Membership::Executors => b"Membership::Executors",
        }
    }
} 

//  Default values for membership is either an buyer or a seller 
impl Default for Membership { 
    fn default() -> Self { 
        Membership::Executors
    }
}

//  Permissions will set the the user's contraints in other pallets 
#[derive(Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Permissions {
    pub pallet_name: Vec<u8>,
    pub type_member: Membership
}