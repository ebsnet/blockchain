extern crate bincode;
extern crate failure;
extern crate sha2;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate blockchain as bc;

mod hack;

pub mod tx;
// pub mod key;

pub use sha2::Sha256;
pub use bc::{block, blockchain};

// use serde::{Deserialize, Serialize};
// use serde::de::Deserialize;

pub const DIFFICULTY: usize = 3;

/// Convenience wrapper for the Blockchain struct.
pub type Blockchain = blockchain::Blockchain<tx::BlockData, Sha256>;
/// Convenience wrapper for the Block struct.
pub type Block = block::Block<tx::BlockData, Sha256>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
