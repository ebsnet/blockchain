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
}

#[derive(Deserialize, Serialize, Clone)]
pub enum Data {
    Billing(String),
    Usage(usize),
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
        Data::Billing("".to_owned())
    }
}

pub trait Signable {
    fn get_bytes(&self) -> Result<Vec<u8>, Error>;
}
