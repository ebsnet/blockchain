//! This module contains the error enumeration for the web service client.

/// Errors that can occur when communicating with the web service.
#[derive(Debug, Fail)]
pub enum ClientError {
    /// Appending a block failed.
    #[fail(display = "Cannot append block")]
    AppendBlock,
    /// Getting the latest block failed.
    #[fail(display = "Cannot get latest block")]
    LatestBlock,
    /// Getting the subchain since the last billing failed.
    #[fail(display = "Cannot get last billing")]
    SinceLastBilling,
    /// A invalid url has been supplied.
    #[fail(display = "Invalid url")]
    InvalidUrl,
}
