use codec::{Encode, Decode};
use frame_support::RuntimeDebug;

use super::*;
//  Executors can trigger dispatch functions
#[derive(Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Permissions { 
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

impl Permissions { 
    fn as_bytes(&self) -> &[u8] { 
        match self { 
            Permissions::Management => b"Permissions::Management",
            Permissions::Executors => b"Permissions::Executors",
        }
    }
} 

//  Default values for membership is either an buyer or a seller 
impl Default for Permissions { 
    fn default() -> Self { 
        Permissions::Executors
    }
}

//  Permissions will set the the user's contraints in other pallets 
#[derive(Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Role {
    pub pallet_name: Vec<u8>,
    pub permission: Permissions
}