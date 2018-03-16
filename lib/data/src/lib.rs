extern crate sha2;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate blockchain as bc;

mod hack;

// pub mod key;
pub mod tx;

pub use sha2::Sha256;

pub use bc::{block, blockchain};

/// Convenience wrapper for the Blockchain struct.
pub type Blockchain = blockchain::Blockchain<tx::Transaction, Sha256>;
/// Convenience wrapper for the Block struct.
pub type Block = block::Block<tx::Transaction, Sha256>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
