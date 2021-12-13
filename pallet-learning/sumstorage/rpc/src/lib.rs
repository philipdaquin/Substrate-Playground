//* Defining the Sum Storage RPC */

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
use sum_storage_runtime_api::SumStorageApi as SumStorageRuntimeApi;


//* The Struct that implements the RPC needs a reference to the client  so we can actually call into the runtime  */
//* Second, the struct is generic over the BlockHash Type, this is because it will call a runtime API 
//* and runtime APIs mist always be a ta  specific block */

#[rpc]
pub trait SumStorageApi<BlockHash> { 
    #[rpc(name = "sumStorage_getSum")]
    fn get_sum(&self, at: Option<BlockHash>) -> Result<u32>;
    #[rpc(name = "sumStorage_getProduct")]
    fn get_product(&self, at: Option<BlockHash>) -> Result<u32>;

}

//  A struct that implements the 'StorageRuntimeApi'
   //  If you have more generics, no need ot SumStorage<C, A, B, C..>
    // just use a tuple like SumStorage<C, (M, N, P, ...)>

pub struct SumStorage<C, M> {  
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> SumStorage<C, M> { 
    pub fn new(client: Arc<C>) -> Self { 
        Self { 
            client,
            _marker: Default::default()
        }
    }
}

pub enum Error { 
    RuntimeError,
    ProductError,
    SumError,
}

impl From<Error> for i64 { 
    fn from(e: Error) -> i64 { 
        match e { 
            Error::RuntimeError => 1,
            Error::ProductError => 2,
            Error::SumError => 3,
        }
    }
}

//* RPC implementation  */

impl<C, Block> SumStorageApi<<Block as BlockT>::Hash> for SumStorage<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: SumStorageRuntimeApi<Block>,
{
    // * Calling the runtime at a specific block, as well as ensuring that the runtime we're calling actually has the correct runtime API available 
    fn get_sum(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u32> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.get_sum(&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
    fn get_product(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u32> { 
        let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.get_sum(&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
    }
}