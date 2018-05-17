#![deny(warnings, missing_docs)]
//! This crate provides an implementation of a `PoW` blockchain and a block, that is generic over the
//! contained data and the used hash algorithm.

extern crate bincode;
extern crate digest;
#[macro_use]
extern crate failure;
extern crate generic_array;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate sha2;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate tempdir;

pub mod blockchain;
pub mod block;

// only used internally. not exposed by the library
mod stack;

/// Enumeration of errors that can occur when working with the blockchain.
#[derive(Debug, Fail)]
pub enum BlockchainError {
    /// The hash of a block does not match its difficulty
    #[fail(display = "invalid block hash \"{}\" with difficulty {}", _0, _1)]
    InvalidBlockHash(String, usize),
    /// The `prev_hash` field does not match the previous block
    #[fail(display = "invalid prev hash \"{}\", should be \"{}\"", _0, _1)]
    InvalidPrevHash(String, String),
    /// An unknown version number.
    #[fail(display = "unknown block version: {}", _0)]
    UnknownVersion(u8),
}

/// Errors that can occur when persisting or loading a blockchain from/to disk.
#[derive(Debug, Fail, Deserialize, Serialize)]
pub enum PersistingError {
    /// Deserialisation failed.
    #[fail(display = "Deserializing failed")]
    DeserializingError,
    /// Serialisation failed.
    #[fail(display = "Serializing failed")]
    SerializingError,
    /// An IO error occurred.
    #[fail(display = "IO error (read/write failed)")]
    IoError,
}

#[cfg(test)]
mod tests {
    #[derive(Default, Serialize, Debug)]
    struct Data {
        foo: u16,
        bar: bool,
    }
}
