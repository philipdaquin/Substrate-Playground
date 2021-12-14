
use super::*;
use crate::types::AttributeId;
use frame_support::{dispatch::DispatchResult, traits::Time};

pub trait Identifier<AccountId, BlockNumber, Moment, Signature> { 
    fn is_owner(
        identity: &AccountId, 
        actual_owner: &AccountId,
    ) -> DispatchResult; 
    
    fn identity_owner(identity: &AccountId) -> AccountId;
    
    fn valid_delegate(
        identity: &AccountId, 
        delegate_type: &[u8],
        delegate: &AccountId, 
    ) -> DispatchResult;

    fn valid_listed_delegate(
        identity: &AccountId,
        delegate_type: &[u8],
        delegate: &AccountId
    ) -> DispatchResult;

    fn create_delegate(
        owner: &AccountId,
        identity: &AccountId,
        delegate: &AccountId,
        delegate_type: &[u8],
        valid_for: Option<BlockNumber>,
    ) -> DispatchResult;

    fn check_signature(
        signature: &Signature, 
        msg: &[u8], 
        signer: &AccountId
    ) -> DispatchResult; 
    
    fn valid_signer(
        identity: &AccountId,
        signature: &Signature,
        msg: &[u8],
        signer: &AccountId,
    ) -> DispatchResult;

    fn create_attribute(
        owner: &AccountId,
        identity: &AccountId, 
        name: &[u8],
        value: &[u8],
        valid_for: Option<BlockNumber>
    ) -> DispatchResult;

    fn reset_attributes(
        owner: AccountId, 
        identity: &AccountId,
        name: &[u8]
    ) -> DispatchResult;

    fn valid_attribute(
        identity: &AccountId, 
        name: &[u8],
        value: &[u8]
    ) -> DispatchResult;

    fn attribute_and_id(
        identity: &AccountId,
        name: &[u8],
    ) ->Option<AttributeId<BlockNumber, Moment>>;

}
