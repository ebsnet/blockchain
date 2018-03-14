use hack::BigArray;

const SIG_SIZE: usize = 64;

pub type SignatureData = [u8; SIG_SIZE];

#[derive(Deserialize, Serialize, Clone)]
pub struct Transaction {
    #[serde(with = "BigArray")]
    signature: SignatureData,
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            signature: [0; SIG_SIZE],
        }
    }
}
