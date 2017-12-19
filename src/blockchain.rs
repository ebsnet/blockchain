use std::fmt::Debug;

use block::Block;

#[derive(Default, Debug)]
pub struct Blockchain<D, H>
where
    D: Default + ::serde::Serialize + Debug + Clone,
    H: ::digest::Digest + Debug,
    <H as ::digest::FixedOutput>::OutputSize: Debug,
{
    blocks: Vec<Block<D, H>>,
}

impl<D, H> Blockchain<D, H>
where
    D: Default + ::serde::Serialize + Debug + Clone,
    H: ::digest::Digest + Debug,
    <H as ::digest::FixedOutput>::OutputSize: Debug,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, block: Block<D, H>) -> bool {
        if self.blocks
            .last()
            .map(|blk| blk.hash() == *block.prev_hash())
            .unwrap_or(true) // last() returns None if Vec is empty -> no prev_hash to verify
            && Self::validate_block(&block)
        {
            self.blocks.push(block);
            true
        } else {
            false
        }
    }

    /// Appends a new block. This method blocks until the given difficulty is reached.
    /// # Examples
    /// ```
    /// extern crate sha2;
    /// # extern crate blockchain;
    /// # fn main() {
    /// use blockchain::blockchain::Blockchain;
    /// let mut bc: Blockchain<_, sha2::Sha256> = Blockchain::new();
    /// bc.append(5, 1); // appends a block with data `5` and difficulty `1` to the chain
    /// # }
    /// ```
    pub fn append(&mut self, data: D, difficulty: u8) -> &Block<D, H> {
        let mut block = Block::new_with_hash(
            data,
            self.blocks.last().map(|blk| blk.hash()).unwrap_or_default(),
            difficulty,
        );
        while !Self::validate_block(&block) {
            block = block.increment_nonce();
        }
        self.blocks.push(block);
        self.blocks.last().unwrap() // Vec cannot be empty -> unwrap never fails
    }

    fn validate_block(block: &Block<D, H>) -> bool {
        let difficulty = block.difficulty();
        block.hash().iter().take(difficulty as usize).all(|&byte| {
            byte == 0
        })
    }

    pub unsafe fn unchecked_append(&mut self, data: D) -> &Block<D, H> {
        let block = Block::new(data, 0);
        self.blocks.push(block);
        self.blocks.last().unwrap()
    }

    pub fn validate_chain(&self) -> bool {
        self.blocks
            .iter()
            .fold((None, true), |acc, blk| { // initial value is None since there is no previous block and true since the chain is valid at that point
                (
                    Some(blk.hash()), // hash of the current block
                    acc.1 && // previous truth value
                        (acc.0) // optional previous block
                            .map(|hash| hash == *blk.prev_hash() && Self::validate_block(blk)) // compare hashes and validate the difficulty
                            .unwrap_or(true), // if there was no previous block, e.g. the genesis block
                )
            })
            .1
    }
}
