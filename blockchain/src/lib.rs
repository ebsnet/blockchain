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

#[derive(Debug, Fail)]
pub enum BlockchainError {
    #[fail(display = "invalid block hash \"{}\" with difficluty {}", _0, _1)]
    InvalidBlockHash(String, usize),
    #[fail(display = "invalid prev hash \"{}\", should be \"{}\"", _0, _1)]
    InvalidPrevHash(String, String),
    #[fail(display = "unknown block version: {}", _0)]
    UnknownVersion(u8),
}

#[derive(Debug, Fail, Deserialize, Serialize)]
pub enum PersistingError {
    #[fail(display = "Deserializing failed")]
    DeserializingError,
    #[fail(display = "Serializeing failed")]
    SerializingError,
    #[fail(display = "IO error (read/write failed)")]
    IoError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;

    #[derive(Default, Serialize, Debug)]
    struct Data {
        foo: u16,
        bar: bool,
    }
}
