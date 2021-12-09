//* Sha3 Proof Of Work Algorithms */

use codec::{Encode, Decode};
use sc_consensus_pow::{Error, PowAlgorithm};
use sha3::{Digest, Sha3_256};
use sp_api::{ProvideRuntimeApi, HashT};
use sp_consensus_pow::{DifficultyApi, Seal as RawSeal};
use sp_core::{H256, U256};
use sp_runtime::generic::BlockId;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;


//  Determine whether the given hash satifies the given difficulty
pub fn hash_difficulty(hash: &H256, difficulty: U256) -> bool { 
    let hash = U256::from(&hash[..]);
    
    let (_, overflow) = hash.overflowing_mul(difficulty);

    !overflow
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug)]
pub struct Compute { 
    pub difficulty: U256,
    pub pre_hash: H256,
    pub nonce: U256
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug)]
pub struct Seal { 
    pub difficulty: U256,
    pub work: H256,
    pub nonce: U256
}

impl Compute { 
    pub fn compute(&self) -> Seal { 
        let work = H256::from_slice(Sha3_256::digest(&self.encode()[..]).as_slice());
        Seal { 
            difficulty: self.difficulty,
            work,
            nonce: self.nonce
        }
    }
}

//  PoW Algorithm that use sha3 for hashing 
//  It needs a reference to the client so it can get the difficulty from the runtime 
#[derive(Clone)]
pub struct ShaAlgorithm<T> { 
    client: Arc<T>
}
impl<T> ShaAlgorithm<T> { 
    pub fn new(client: Arc<T>) -> Self { 
        Self { 
            client
        }
    }
}

///  Implement the general Pow Algorithm trait
///     Difficulty is fixed, this means that as more mining power joins the network, 
///     the block time will become faster  
impl<A: BlockT<Hash = H256>, T> PowAlgorithm<A> for ShaAlgorithm<T> 
    where T: ProvideRuntimeApi<A>, T::Api: DifficultyApi<A, U256>
{ 
    type Difficulty = U256;
    //  
    fn difficulty(&self, parent: A::Hash) -> Result<Self::Difficulty, Error<A>> { 
        let parent_id = BlockId::<A>::hash(parent);

        self.client
            .runtime_api()
            .difficulty(&parent_id)
            .map_err(|e| { 
                sc_consensus_pow::Error::Environment(format! { 
                    "Difficulty from Runtime Failed: {:?}", e
                })
            })
    }
    /// prehash: the hash of the block before the pow seal is attached 
    /// seal: checks whether the work has been done
    /// difficulty: what diffuclty did the author completed this 
    /// 
    fn verify(&self, 
        parent: &BlockId<A>, 
        pre_hash: &H256, 
        pre_digest: Option<&[u8]>,
        seal: &RawSeal,
        difficulty: Self::Difficulty
    ) -> Result<bool, Error<A>> { 
        //  Check that the seal actually meets the target difficulty 
        let seal = {
            if let Ok(seal) = Seal::decode(&mut &seal[..]) { 
                seal
            } else { 
                return Ok(false)
            }
        };
        if !hash_difficulty(&seal.work, difficulty) { 
            return Ok(false)
        }
        let compute = Compute { 
            difficulty,
            pre_hash: *pre_hash,
            nonce: seal.nonce
        };

        if compute.compute() != seal { 
            return Ok(false)
        }
        Ok(true)
    }
}

//  Very Simple Implementation of Proof of Work 
#[derive(Clone)]
pub struct SimplePoWAlgo;

impl<X: BlockT<Hash = H256>> PowAlgorithm<X> for SimplePoWAlgo { 
    type Difficulty = U256;

    fn difficulty(&self, parent: X::Hash) -> Result<Self::Difficulty, Error<X>> { 
        //  Fixed Difficulty at 1_000_000
        Ok(U256::from(1_000_000))

    }
    /// prehash: the hash of the block before the pow seal is attached 
    /// seal: checks whether the work has been done
    /// difficulty: what diffuclty did the author completed this 
    /// 
    fn verify(&self, 
        parent: &BlockId<X>, 
        pre_hash: &H256, 
        pre_digest: Option<&[u8]>,
        seal: &RawSeal,
        difficulty: Self::Difficulty
    ) -> Result<bool, Error<X>> { 
        //  Check that the seal actually meets the target difficulty 
        let seal = {
            if let Ok(seal) = Seal::decode(&mut &seal[..]) { 
                seal
            } else { 
                return Ok(false)
            }
        };
        if !hash_difficulty(&seal.work, difficulty) { 
            return Ok(false)
        }
        let compute = Compute { 
            difficulty,
            pre_hash: *pre_hash,
            nonce: seal.nonce
        };

        if compute.compute() != seal { 
            return Ok(false)
        }
        Ok(true)
    }

}