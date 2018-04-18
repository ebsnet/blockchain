//! Due to the way, server state is handled by rocket, we need a wrapper class around the
//! functional implementation of the blockchain and work with impure functions.

use data::{BcIter, Block, Blockchain};

use error::BlockchainError;

/// Impure wrapper for the blockchain.
pub struct WrappedChain {
    chain: Blockchain,
}

impl WrappedChain {
    /// Wraps a blockchain.
    pub fn new(chain: Blockchain) -> Self {
        Self { chain: chain }
    }

    /// Append a new block to the chain by modifying the struct (impure).
    pub fn append(&mut self, block: Block, path: &str) -> Result<(), BlockchainError> {
        if let Ok(new) = self.chain.insert(block) {
            self.chain = new;
            self.chain.persist_to_disk(path).ok();
            Ok(())
        } else {
            Err(BlockchainError::InvalidBlock)
        }
    }

    /// Returns a copy of the latest block.
    pub fn latest_block(&self) -> Option<Block> {
        self.chain.tail().0.cloned()
    }

    pub fn iter(&self) -> BcIter {
        self.chain.iter()
    }
}
