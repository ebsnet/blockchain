use std::ops::DerefMut;
use std::sync::RwLock;

use error::BlockchainError;
use data::{Block, Blockchain};
use wrapper::WrappedChain;

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
