use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_std::vec::Vec;
use super::*;
//  Attributes or properties that make an identity 
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, scale_info::TypeInfo, Decode, Default, RuntimeDebug)]
pub struct Attribute<BlockNumber, Moment> { 
    pub name: Vec<u8>,
    pub value: Vec<u8>,
    pub validity: BlockNumber,
    pub creation: Moment, 
    pub nonce: u64
}

pub type AttributeId<BlockNumber, Moment> = (Attribute<BlockNumber, Moment>, [u8; 32]);

//  Offchain signed transaction 
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct AttributeTransaction<Signature, AccountId> { 
    pub signature: Signature,
    pub name: Vec<u8>,
    pub value: Vec<u8>,
    pub validity: u32, 
    pub signer: AccountId, 
    pub identity: AccountId
}
 