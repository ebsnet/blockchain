#[derive(Debug, Fail)]
pub enum ClientError {
    #[fail(display = "Cannot append block")]
    AppendBlock,
    #[fail(display = "Cannot get latest block")]
    LatestBlock,
    #[fail(display = "Cannot get latest billing")]
    SinceLatestBilling,
    #[fail(display = "Invalid url")]
    InvalidUrl,
}
