use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use super::{BlockchainError, PersistingError};

use serde::ser::{Serialize, Serializer};
use serde::de::{Deserialize, Deserializer};

use block::{Block, current_time};
use stack::Stack;

#[derive(Debug, Clone)]
pub struct Blockchain<D, H>
where
    H: ::digest::Digest,
    <H as ::digest::FixedOutput>::OutputSize: Debug + Clone,
{
    blocks: Stack<Block<D, H>>,
}

// Clippy warns on missing `is_empty` method if a method `len` is available, since in many cases
// `is_empty` might be implemented more efficient than `len`. Since the stack that is used for this
// blockchain implements `len` in `O(1)`, this is not necessary.
// https://rust-lang-nursery.github.io/rust-clippy/current/index.html#len_without_is_empty
#[cfg_attr(feature = "cargo-clippy", allow(len_without_is_empty))]
impl<D, H> Blockchain<D, H>
where
    D: Default,
    H: ::digest::Digest,
    <H as ::digest::FixedOutput>::OutputSize: Debug + Clone,
{
    /// Creates a new and empty blockchain.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::blockchain::Blockchain;
    /// let bc: Blockchain<bool, sha2::Sha256> = Blockchain::new();
    /// assert_eq!(bc.len(), 0);
    /// # }
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the length of the blockchain.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::blockchain::Blockchain;
    /// let bc: Blockchain<_, sha2::Sha256> = Blockchain::new();
    /// assert_eq!(bc.len(), 0);
    /// let bc = unsafe { bc.unchecked_append(3) };
    /// assert_eq!(bc.len(), 1);
    /// # }
    /// ```
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Removes the latest block from the blockchain. Returns an optional reference to the removed
    /// block and a new blockchain object.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::blockchain::Blockchain;
    /// let bc: Blockchain<_, sha2::Sha256> = Blockchain::new();
    /// let (head, bc) = bc.tail();
    /// assert_eq!(head, None);
    /// assert_eq!(bc.len(), 0);
    /// let bc = bc.append(42, 0);
    /// let (head, bc) = bc.tail();
    /// assert_eq!(head.map(|b| b.data()), Some(&42));
    /// assert_eq!(bc.len(), 0);
    /// # }
    /// ```
    pub fn tail(&self) -> (Option<&Block<D, H>>, Blockchain<D, H>) {
        let tail = self.blocks.tail();
        (tail.0, Self { blocks: tail.1 })
    }
    /// Appends a new block with difficulty 0 and an empty previous hash to the chain without
    /// checking. This method is unsafe in a logical sense and therefore marked as unsafe in the
    /// Rust sense. It might corrupt your blockchain, use with caution. For production use, you
    /// should use [`insert(&self, block: Block<D, H>)`](#method.insert).
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::blockchain::Blockchain;
    /// let bc: Blockchain<_, sha2::Sha256> = Blockchain::new();
    /// let bc = unsafe { bc.unchecked_append(3) };
    /// let bc = unsafe { bc.unchecked_append(5) };
    /// assert!(!bc.validate_chain());
    /// # }
    /// ```
    pub unsafe fn unchecked_append(&self, data: D) -> Self {
        let block = Block::new(data, 0);
        Self { blocks: self.blocks.append(block) }
    }

    /// Creates an iterator over the blockchain, that iterates the chain in reverse order (newest
    /// block first).
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::blockchain::Blockchain;
    /// let bc: Blockchain<_, sha2::Sha256> = Blockchain::new();
    /// let bc = bc.append(5, 0);
    /// let bc = bc.append(42, 1);
    /// let mut iter = bc.iter();
    /// assert_eq!(iter.next().unwrap().data(), &42);
    /// assert_eq!(iter.next().unwrap().data(), &5);
    /// assert_eq!(iter.next(), None);
    /// # }
    /// ```
    pub fn iter(&self) -> ::stack::Iter<Block<D, H>> {
        self.blocks.iter()
    }
}

impl<D, H> Blockchain<D, H>
where
    D: Default + Serialize,
    H: ::digest::Digest,
    <H as ::digest::FixedOutput>::OutputSize: Debug + Clone,
{
    /// Validates the blockchain. Checks if each block contains the hash of the previous block and
    /// if the hash of a block matches its difficulty.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::blockchain::Blockchain;
    /// let bc: Blockchain<_, sha2::Sha256> = Blockchain::new();
    /// assert!(bc.validate_chain());
    /// let bc = bc.append(5, 1); // appends a block with data `5` and difficulty `1` to the chain
    /// assert!(bc.validate_chain());
    /// let bc = bc.append(42, 1);
    /// assert!(bc.validate_chain());
    /// # }
    /// ```
    pub fn validate_chain(&self) -> bool {
        self.iter()
            .fold((None, true), |acc, blk| {
                (
                    Some(blk),
                    acc.1 &&
                        acc.0
                            .map(|b| {
                                *b.prev_hash() == blk.hash() && b.time() >= blk.time() &&
                                    Self::validate_block(blk).is_ok()
                            })
                            .unwrap_or(true),
                )
            })
            .1
    }

    /// Appends a new block to the blockchain. The block gets validated and if validation fails an
    /// error is returned. If the block is valid, a new head of the chain is returned.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::blockchain::Blockchain;
    /// let bc: Blockchain<_, sha2::Sha256> = Blockchain::new();
    /// let block0 = bc.generate_block(42, 1);
    /// let bc = bc.insert(block0);
    /// assert!(bc.is_ok());
    /// let bc = bc.unwrap().insert(Default::default()); // insertion if invalid block
    /// assert!(bc.is_err());
    /// # }
    /// ```
    pub fn insert(&self, block: Block<D, H>) -> Result<Self, BlockchainError> {
        self.blocks
            .head()
            .map_or(Ok(()), |head| if head.hash() == *block.prev_hash() {
                Ok(())
            } else {
                Err(BlockchainError::InvalidPrevHash(
                    format!("{:?}", block.prev_hash()),
                    format!("{:?}", head.hash()),
                ))
            })
            .and_then(|_| Self::validate_block(&block))
            .map(|_| Self { blocks: self.blocks.append(block) })
    }

    /// Generates a new block ready to append to the blockchain. The block will contain the hash of
    /// the previous block from the chain and a nonce that `hash(block)` matches the given
    /// difficulty. This method blocks the current thread until the block's hash matches the
    /// difficulty. This might take a very long time.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::blockchain::Blockchain;
    /// let bc: Blockchain<_, sha2::Sha256> = Blockchain::new();
    /// let block0 = bc.generate_block(42, 1);
    /// let bc = bc.insert(block0.clone());
    /// assert!(bc.is_ok());
    /// let bc = bc.unwrap();
    /// let block1 = bc.generate_block(1337, 1);
    /// let bc = bc.insert(block1);
    /// assert!(bc.is_ok());
    /// let bc = bc.unwrap().insert(block0); // after inserting block1, block0 is no longer valid
    /// assert!(bc.is_err());
    /// # }
    /// ```
    pub fn generate_block(&self, data: D, difficulty: u8) -> Block<D, H> {
        let mut block = Block::new_with_hash(
            data,
            self.blocks.head().map(|blk| blk.hash()).unwrap_or_default(),
            difficulty,
        );
        while Self::validate_block(&block).is_err() {
            block = block.increment_nonce(current_time());
        }
        block
    }

    /// Appends a new block. This method blocks until the given difficulty is reached.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::blockchain::Blockchain;
    /// let bc: Blockchain<_, sha2::Sha256> = Blockchain::new();
    /// let bc = bc.append(5, 1); // appends a block with data `5` and difficulty `1` to the chain
    /// # }
    /// ```
    pub fn append(&self, data: D, difficulty: u8) -> Blockchain<D, H> {
        self.insert(self.generate_block(data, difficulty)).expect(
            "This cannot happen!",
        ) // this cannot fail since we just created a valid block
    }

    fn validate_block(block: &Block<D, H>) -> Result<(), BlockchainError> {
        let difficulty = block.difficulty();
        let valid_difficulty = block.hash().iter().take(difficulty as usize).all(|&byte| {
            byte == 0
        });
        if block.version() != ::block::VERSION {
            Err(BlockchainError::UnknownVersion(block.version()))
        } else if !valid_difficulty {
            Err(BlockchainError::InvalidBlockHash(
                format!("{:?}", block.hash()),
                block.difficulty(),
            ))
        } else {
            Ok(())
        }
    }
}

impl<D, H> Blockchain<D, H>
where
    D: Serialize,
    H: ::digest::Digest,
    <H as ::digest::FixedOutput>::OutputSize: Debug + Clone,
{
    pub fn persist_to_disk<P: AsRef<Path>>(&self, filename: P) -> Result<(), ::failure::Error> {
        let encoded: Vec<u8> = ::bincode::serialize(self, ::bincode::Infinite)?;
        let mut file = BufWriter::new(File::create(filename)?);
        file.write_all(&encoded).map_err(From::from)
    }
}

impl<D, H> Blockchain<D, H>
where
    D: Serialize,
    for<'de> D: Deserialize<'de>,
    H: ::digest::Digest,
    <H as ::digest::FixedOutput>::OutputSize: Debug + Clone,
{
    pub fn load_from_disk<P: AsRef<Path>>(filename: P) -> Result<Self, PersistingError> {
        let mut file = BufReader::new(File::open(filename).map_err(|_| PersistingError::IoError)?);
        ::bincode::deserialize_from(&mut file, ::bincode::Infinite)
            .map_err(|_| PersistingError::DeserializingError)
    }
}

impl<D, H> Default for Blockchain<D, H>
where
    D: Default,
    H: ::digest::Digest,
    <H as ::digest::FixedOutput>::OutputSize: Debug + Clone,
{
    fn default() -> Self {
        Self { blocks: Default::default() }
    }
}

impl<D, H> ::serde::Serialize for Blockchain<D, H>
where
    D: ::serde::Serialize,
    H: ::digest::Digest,
    <H as ::digest::FixedOutput>::OutputSize: Debug + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.blocks.serialize(serializer)
    }
}

impl<'de, D, H> Deserialize<'de> for Blockchain<D, H>
where
    D: Deserialize<'de> + Serialize,
    H: ::digest::Digest,
    <H as ::digest::FixedOutput>::OutputSize: Debug + Clone,
{
    fn deserialize<S>(deserializer: S) -> Result<Self, S::Error>
    where
        S: Deserializer<'de>,
    {
        Ok(Self { blocks: Stack::deserialize(deserializer)? })
    }
}

impl<D, H> PartialEq for Blockchain<D, H>
where
    D: PartialEq,
    H: ::digest::Digest,
    <H as ::digest::FixedOutput>::OutputSize: Debug + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.blocks == other.blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl<A> Arbitrary for Blockchain<A, ::sha2::Sha256>
    where
        A: Arbitrary
            + ::std::marker::Sync
            + Default
            + ::serde::Serialize,
        for<'de> A: Deserialize<'de>,
    {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let size = {
                let s = g.size();
                g.gen_range(0, s)
            };
            (0..size).fold(Blockchain::new(), |acc, _| {
                acc.append(Arbitrary::arbitrary(g), 0)
            })
        }
    }

    quickcheck! {
        fn persist_and_load_is_equal(xs: Blockchain<bool, ::sha2::Sha256>) -> bool {
            if let Ok(dir) = ::tempdir::TempDir::new("blockchain_") {
                let file_name = dir.path().join("chain.bin");
                xs.persist_to_disk(&file_name).unwrap();
                let new_chain = Blockchain::load_from_disk(&file_name).unwrap();
                dir.close().unwrap();
                xs == new_chain
            } else {
                false
            }
        }
    }

    quickcheck! {
         fn append_results_in_valid_chain(chain: Blockchain<bool, ::sha2::Sha256>) -> bool {
             let chain = chain.append(false, 1);
             chain.validate_chain()
         }
     }
}
