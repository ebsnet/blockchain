//! This module contains transaction specific data.

use hack::BigArray;

use bincode::serialize;

use failure::Error;

/// Size of a Ed25519 signature in bytes.
pub const SIG_SIZE: usize = 64;

/// Convenience type for a signature.
pub type Signature = [u8; SIG_SIZE];

/// Convenience type for signed data inside a block.
pub type BlockData = SignedData<Data>;

/// Wrapper for signed date. This struct contains the data and the signature.
#[derive(Deserialize, Serialize, Clone)]
pub struct SignedData<T> {
    #[serde(with = "BigArray")]
    signature: Signature,
    data: T,
}

impl<T> SignedData<T> {
    /// Generate a new object from supplied data and a signature.
    pub fn new(signature: Signature, data: T) -> Self {
        Self {
            signature: signature,
            data: data,
        }
    }

    /// Returns a reference to the wrapped data.
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns a reference to the wrapped signature.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

/// Convenience type for a fingerprint.
pub type Fingerprint = Vec<u8>;

/// The data that can be contained in a block.
#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub enum Data {
    /// Billing operation used to initialize a billing process and indicate that a user has been
    /// billed at a certain point in time.
    Billing(Fingerprint),
    /// Usage operation that protocols the power usage of a user.
    Usage(u64),
}

/// Typed that implement this trait can be signed.
pub trait Signable {
    /// Converts the data to a list of bytes that can be signed.
    fn get_bytes(&self) -> Result<Vec<u8>, Error>;
}

impl Signable for Data {
    fn get_bytes(&self) -> Result<Vec<u8>, Error> {
        let res = serialize(self)?;
        Ok(res)
    }
}

impl<T> Default for SignedData<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            signature: [0; SIG_SIZE],
            data: Default::default(),
        }
    }
}

impl Default for Data {
    fn default() -> Self {
        Data::Billing(Default::default())
    }
}
