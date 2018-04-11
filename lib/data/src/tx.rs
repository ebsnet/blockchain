use hack::BigArray;

use bincode::serialize;

use failure::Error;

pub const SIG_SIZE: usize = 64;

pub type Signature = [u8; SIG_SIZE];

pub type BlockData = SignedData<Data>;

#[derive(Deserialize, Serialize, Clone)]
pub struct SignedData<T> {
    #[serde(with = "BigArray")]
    signature: Signature,
    data: T,
}

impl<T> SignedData<T> {
    pub fn new(signature: Signature, data: T) -> Self {
        Self {
            signature: signature,
            data: data,
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

pub type Fingerprint = Vec<u8>;

#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub enum Data {
    Billing(Fingerprint),
    Usage(u64),
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

pub trait Signable {
    fn get_bytes(&self) -> Result<Vec<u8>, Error>;
}
