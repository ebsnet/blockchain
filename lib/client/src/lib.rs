extern crate data;
#[macro_use]
extern crate failure;
extern crate reqwest;

pub mod error;

use error::ClientError;

use failure::Error;

use data::Block;

use reqwest::StatusCode;

// http routes for the webservice

const ROUTE_LATEST_BLOCK: &str = "/latest_block";
const ROUTE_APPEND: &str = "/append";

pub struct Client<'a> {
    client: reqwest::Client,
    host: &'a str,
}

impl<'a> Client<'a> {
    pub fn new(host: &'a str) -> Self {
        Self {
            client: reqwest::Client::new(),
            host: host,
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
