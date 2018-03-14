use hack::BigArray;

const KEY_SIZE: usize = 85;

pub type KeyData = [u8; KEY_SIZE];

#[derive(Deserialize, Serialize)]
pub struct Key {
    #[serde(with = "BigArray")]
    key: KeyData,
}
