#[derive(Debug, Fail)]
pub enum ClientError {
    #[fail(display = "Cannot append block")]
    AppendBlock,
    #[fail(display = "Cannot get latest block")]
    LatestBlock,
}
