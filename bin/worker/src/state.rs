use std::ops::{Deref, DerefMut};
use std::sync::RwLock;

use error::BlockchainError;
use data::{Block, Blockchain};
use data::tx::Data;
use wrapper::WrappedChain;
use cryptography::{validate_signature, BillingQuery};

pub struct ServerState {
    chain: RwLock<WrappedChain>,
    path: String,
}

impl ServerState {
    pub fn new(chain: Blockchain, path: String) -> Self {
        Self {
            chain: RwLock::new(WrappedChain::new(chain)),
            path: path,
        }
    }

    pub fn latest_billing(
        &self,
        query: BillingQuery,
    ) -> Result<Option<Blockchain>, BlockchainError> {
        if let Ok(chain) = self.chain.read() {
            let chain = chain.deref();
            let mut cloned = Vec::new();
            for blk in chain.iter() {
                cloned.push(blk.clone());
                let blockdata = blk.data();
                if match *blockdata.data() {
                    Data::Billing(ref fp) => {
                        fp == query.user()
                            && validate_signature(query.signee(), blockdata).unwrap_or(false)
                    }
                    _ => false,
                } {
                    break;
                }

                // reached the genesis block and did not find billing operation
                if blk.is_genesis() {
                    return Ok(None);
                }
            }
            cloned.reverse();
            Ok(cloned
                .into_iter()
                .fold(Ok(Blockchain::new()), |acc, blk| {
                    acc.and_then(|chain| chain.insert(blk))
                })
                .ok())
        } else {
            Err(BlockchainError::CannotGetLock)
        }
    }

    pub fn append(&self, block: Block, path: &str) -> Result<(), BlockchainError> {
        if let Ok(mut chain) = self.chain.write() {
            chain.deref_mut().append(block, path)
        } else {
            Err(BlockchainError::CannotGetLock)
        }
    }

    pub fn latest_block(&self) -> Result<Block, BlockchainError> {
        if let Ok(chain) = self.chain.read() {
            chain.latest_block().ok_or(BlockchainError::EmptyChain)
        } else {
            Err(BlockchainError::CannotGetLock)
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
