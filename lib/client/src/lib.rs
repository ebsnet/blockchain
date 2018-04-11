extern crate cryptography;
extern crate data;
#[macro_use]
extern crate failure;
extern crate reqwest;

pub mod error;

use error::ClientError;

use data::{Block, Blockchain};

use cryptography::BillingQuery;

use reqwest::StatusCode;

// http routes for the webservice

const ROUTE_LATEST_BLOCK: &str = "/latest_block";
const ROUTE_APPEND: &str = "/append";
const ROUTE_LATEST_BILLING: &str = "/since_latest_billing";

pub struct Client<'a> {
    client: reqwest::Client,
    host: &'a str,
}

impl<'a> Client<'a> {
    pub fn new(host: &'a str) -> Result<Self, ClientError> {
        if !(host.starts_with("http://") || host.starts_with("https://")) {
            Err(ClientError::InvalidUrl)
        } else {
            Ok(Self {
                client: reqwest::Client::new(),
                host: host,
            })
        }
    }

    pub fn latest_block(&self) -> Result<Block, ClientError> {
        self.client
            .get(&format!("{}{}", self.host, ROUTE_LATEST_BLOCK))
            .send()
            .and_then(|mut response| response.json())
            .map_err(|_| ClientError::LatestBlock)
    }

    pub fn append(&self, block: &Block) -> Result<(), ClientError> {
        self.client
            .post(&format!("{}{}", self.host, ROUTE_APPEND))
            .json(block)
            .send()
            .map_err(|_| ClientError::AppendBlock)
            .and_then(|r| {
                if r.status() == StatusCode::Accepted {
                    Ok(())
                } else {
                    Err(ClientError::AppendBlock)
                }
            })
    }

    pub fn since_latest_billing(
        &self,
        query: &BillingQuery,
    ) -> Result<Option<Blockchain>, ClientError> {
        self.client
            .post(&format!("{}{}", self.host, ROUTE_LATEST_BILLING))
            .json(query)
            .send()
            .and_then(|mut resp| resp.json())
            .map_err(|_| ClientError::SinceLatestBilling)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
