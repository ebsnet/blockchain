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
pub mod blockchain_imut;
pub mod block;

mod chain;

#[derive(Debug, Fail)]
pub enum BlockchainError {
    #[fail(display = "invalid block hash \"{}\" with difficluty {}", _0, _1)]
    InvalidBlockHash(String, u8),
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

    // #[test]
    fn blockchain_3_blocks() {
        use blockchain::Blockchain;

        let mut bc: Blockchain<_, ::sha2::Sha256> = Blockchain::new();
        writeln!(std::io::stderr(), "valid: {:?}", bc.validate_chain()).unwrap();
        {
            let block = bc.append(5, 2);
            // let block = bc.append(Data { foo: 3, bar: false }, 2);
            writeln!(std::io::stderr(), "first block: {:?}", block).unwrap();
            writeln!(std::io::stderr(), "first hash: {:?}", block.hash()).unwrap();
        }
        {
            let block = bc.append(10, 2);
            // let block = bc.append(Data { foo: 2, bar: false }, 2);
            writeln!(std::io::stderr(), "second block: {:?}", block).unwrap();
            writeln!(std::io::stderr(), "second hash: {:?}", block.hash()).unwrap();
        }
        {
            let block = bc.append(20, 2);
            // let block = bc.append(Data { foo: 5, bar: true }, 2);
            writeln!(std::io::stderr(), "third block: {:?}", block).unwrap();
            writeln!(std::io::stderr(), "third hash: {:?}", block.hash()).unwrap();
        }
        writeln!(std::io::stderr(), "valid: {:?}", bc.validate_chain()).unwrap();
        unsafe {
            let block = bc.unchecked_append(3);
            writeln!(std::io::stderr(), "fourth (unchecked) block: {:?}", block).unwrap();
        }
        writeln!(std::io::stderr(), "valid: {:?}", bc.validate_chain()).unwrap();
    }
}
