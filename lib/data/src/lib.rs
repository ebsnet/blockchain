#![deny(warnings, missing_docs)]
//! This crate exports a specific blockchain and data structures for blocks.

extern crate bincode;
extern crate failure;
extern crate sha2;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate blockchain as bc;

mod hack;

pub mod tx;

pub use sha2::Sha256;
pub use bc::{block, blockchain};

/// The difficulty factor.
pub const DIFFICULTY: usize = 3;

/// Convenience type for the Blockchain struct.
pub type Blockchain = blockchain::Blockchain<tx::BlockData, Sha256>;
/// Convenience type for the Block struct.
pub type Block = block::Block<tx::BlockData, Sha256>;
/// Iterator over the specific blockchain.
pub type BcIter<'a> = blockchain::BlockchainIter<'a, tx::BlockData, Sha256>;
