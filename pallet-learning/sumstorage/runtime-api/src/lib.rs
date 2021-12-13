#![cfg_attr(not(feature = "std"), no_std)]
#[warn(clippy::too_many_arguments)]
#[warn(clippy::unnecessary_mut_passed)]
///  Defining the API 
/// This macro allows the outer node to query the runtime API at specific blocks
sp_api::decl_runtime_apis! { 
    pub trait SumStorageApi { 
        fn get_sum() -> u32;
        fn get_product() -> u32;
    }
    
}