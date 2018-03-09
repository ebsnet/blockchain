const SIG_SIZE: usize = 64;

pub struct Transaction {
    signature: [u8; SIG_SIZE],
}
