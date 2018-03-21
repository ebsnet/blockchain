use std::time::{SystemTime, UNIX_EPOCH};

use generic_array::GenericArray;
use generic_array::typenum::Unsigned;

pub const VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block<D, H>
where
    H: ::digest::Digest,
{
    version: u8,
    prev_hash: GenericArray<u8, H::OutputSize>,
    time: u64,
    difficulty: usize,
    nonce: u64,
    data: D,
}

impl<D, H> Block<D, H>
where
    D: Default,
    H: ::digest::Digest,
{
    /// Creates a new block with the given data and difficulty. The nonce is initialized to `0` and
    /// an empty `prev_hash` is set so the block is in an invalid state. The time field is
    /// initialized with the current Unix timestamp.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::block::Block;
    /// let block: Block<_, ::sha2::Sha256> = Block::new(5, 1);
    /// assert_eq!(block.data(), &5);
    /// assert_eq!(block.difficulty(), 1);
    /// # }
    /// ```
    pub fn new(data: D, difficulty: usize) -> Self {
        Self::assert_difficulty(difficulty);
        Self {
            data: data,
            difficulty: difficulty,
            ..Default::default()
        }
    }

    /// Creates a new block with the given data, `prev_hash` and difficulty. The nonce is
    /// initialized to `0`, so the block might be in an invalid state, since its own hash doesn't
    /// match the difficulty. The time field is initialized with the current Unix timestamp.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// extern crate generic_array;
    /// extern crate digest;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use generic_array::GenericArray;
    /// use blockchain::block::Block;
    /// let mut hash = Block::<_, ::sha2::Sha256>::new(5, 1).hash();
    /// hash[0] = 0xff;
    /// let block: Block<_, ::sha2::Sha256> = Block::new_with_hash(5, hash, 1);
    /// assert_eq!(block.data(), &5);
    /// assert_eq!(block.difficulty(), 1);
    /// assert_eq!(block.prev_hash(), &hash);
    /// # }
    /// ```
    pub fn new_with_hash(
        data: D,
        prev_hash: GenericArray<u8, H::OutputSize>,
        difficulty: usize,
    ) -> Self {
        Self::assert_difficulty(difficulty);
        Self {
            data: data,
            prev_hash: prev_hash,
            difficulty: difficulty,
            ..Default::default()
        }
    }

    pub fn hash_length_byte() -> usize {
        H::OutputSize::to_usize()
    }

    pub fn hash_length_bit() -> usize {
        Self::hash_length_byte() * 8
    }

    #[inline]
    fn assert_difficulty(difficulty: usize) {
        assert!(
            Self::hash_length_bit() >= difficulty,
            "Difficulty cannot be larger than the hash length"
        );
    }
}

impl<D, H> Block<D, H>
where
    H: ::digest::Digest,
{
    /// Returns the block version.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::block::{Block, VERSION};
    /// let block: Block<_, ::sha2::Sha256> = Block::new(42, 1);
    /// assert_eq!(block.version(), VERSION);
    /// # }
    /// ```
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns the difficulty of a block.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::block::Block;
    /// let block: Block<_, ::sha2::Sha256> = Block::new(42, 1);
    /// assert_eq!(block.difficulty(), 1);
    /// # }
    /// ```
    pub fn difficulty(&self) -> usize {
        self.difficulty
    }

    /// Returns the time, a block was created or last modified as a Unix timestamp. The time is
    /// updated every time a value in a block is changed.
    pub fn time(&self) -> u64 {
        self.time
    }

    /// Returns a reference to the data inside a block.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::block::Block;
    /// let block: Block<_, ::sha2::Sha256> = Block::new(42, 1);
    /// assert_eq!(block.data(), &42);
    /// # }
    /// ```
    pub fn data(&self) -> &D {
        &self.data
    }

    /// Returns the hash of the previous block.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::block::Block;
    /// let block0: Block<_, ::sha2::Sha256> = Block::new(42, 1);
    /// let block1: Block<_, ::sha2::Sha256> = Block::new_with_hash(43, block0.hash(), 1);
    /// assert_eq!(&block0.hash(), block1.prev_hash());
    /// # }
    /// ```
    pub fn prev_hash(&self) -> &GenericArray<u8, H::OutputSize> {
        &self.prev_hash
    }

    /// Returns the nonce.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::block::Block;
    /// let block: Block<_, ::sha2::Sha256> = Block::new(42, 1);
    /// assert_eq!(block.nonce(), 0);
    /// # }
    /// ```
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Sets the nonce to an arbitrary value and sets the the `time` attribute. The method returns
    /// a new block and consumes the old one.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::block::{Block, current_time};
    /// let block: Block<_, ::sha2::Sha256> = Block::new(42, 1);
    /// let block = block.set_nonce(1337, 0);
    /// assert_eq!(block.nonce(), 1337);
    /// # }
    /// ```
    pub fn set_nonce(self, nonce: u64, time: u64) -> Self {
        Self {
            nonce: nonce,
            time: time,
            ..self
        }
    }

    /// Increments the nonce by 1 and updates the `time` attribute. The method returns a new block
    /// and consumes the old one. If the nonce overflows, it will start again at 0.
    ///
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::block::Block;
    /// let block: Block<_, ::sha2::Sha256> = Block::new(42, 1);
    /// assert_eq!(block.nonce(), 0);
    /// let block = block.increment_nonce(0);
    /// assert_eq!(block.nonce(), 1);
    /// let block = block.set_nonce(u64::max_value(), 0);
    /// let block = block.increment_nonce(0);
    /// assert_eq!(block.nonce(), 0);
    /// # }
    pub fn increment_nonce(self, time: u64) -> Self {
        let nonce = self.nonce;
        self.set_nonce(nonce.wrapping_add(1), time)
    }
}

impl<D, H> Block<D, H>
where
    D: ::serde::Serialize,
    H: ::digest::Digest,
{
    pub fn as_bytes(&self) -> Vec<u8> {
        ::bincode::serialize(self, ::bincode::Infinite).unwrap()
    }

    pub fn hash(&self) -> GenericArray<u8, H::OutputSize> {
        H::digest(&self.as_bytes())
    }

    pub fn validate_difficulty(&self) -> bool {
        self.hash()
            .iter()
            .take((self.difficulty / 8) + 1)
            .fold((self.difficulty, true), |(d, b), byte| {
                let leading_zeros = byte.leading_zeros();
                if d >= 8 {
                    (d - 8, b && leading_zeros == 8)
                } else {
                    (d, b && leading_zeros >= d as u32)
                }
            })
            .1
    }

    pub fn proof_of_work(self) -> Self {
        if self.validate_difficulty() {
            self
        } else {
            self.increment_nonce(current_time()).proof_of_work()
        }
    }
}

/// Returns the time in seconds since `1970-01-01`.
pub fn current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()   // TODO: don't unwrap()
        .as_secs()
}

impl<D, H> Default for Block<D, H>
where
    D: Default,
    H: ::digest::Digest,
{
    fn default() -> Self {
        Self {
            version: VERSION,
            prev_hash: Default::default(),
            difficulty: 0,
            nonce: 0,
            time: current_time(),
            data: Default::default(),
        }
    }
}

impl<D, H> PartialEq for Block<D, H>
where
    D: PartialEq,
    H: ::digest::Digest,
{
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version && self.prev_hash == other.prev_hash
            && self.difficulty == other.difficulty && self.nonce == other.nonce
            && self.time == other.time && self.data == other.data
    }
}
