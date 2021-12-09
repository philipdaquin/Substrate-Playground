use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

#[rpc]
pub trait SillyRpc { 
    #[rpc(name = "monkey")]
    fn monkey(&self) -> Result<u64>;

    #[rpc(name = "monkey_double")]
    fn monkey_double(&self, val: u64) -> Result<u64>; 
}

//  Custom SillyRpc 
pub struct Silly;

impl SillyRpc for Silly { 
    fn monkey(&self) -> Result<u64> { 
        self
    }
    fn monkey_double(&self, val: u64) -> Result<u64> { 
        Ok(2 * val)
    }
}