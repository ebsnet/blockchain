#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate clap;

#[macro_use]
extern crate failure;

extern crate ring;
extern crate untrusted;

extern crate rocket;
extern crate rocket_contrib;

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate cryptography;
extern crate data;

mod cli;
mod error;
mod server;
mod state;
mod wrapper;

use error::BlockchainError;
use state::ServerState;

use data::blockchain;

/// Default path to look for the blockchain.
const DEFAULT_BC_PATH: &str = "./blockchain.dat";
/// Default path for the webserver to listen on.
const DEFAULT_PORT: &str = "1337";
/// Default address for the webserver to listen on.
const DEFAULT_ADDRESS: &str = "localhost";

fn main() {
    env_logger::init();
    let matches = cli::build_cli();

    let data_path = matches.value_of("BLOCKCHAIN").unwrap_or(DEFAULT_BC_PATH);
    let blockchain = ServerState::new(
        blockchain::Blockchain::load_from_disk(data_path).unwrap_or_default(),
        data_path.to_owned(),
    );

    let port = matches
        .value_of("PORT")
        .unwrap_or(DEFAULT_PORT)
        .parse()
        .expect("Cannot parse port");

    let address = matches.value_of("ADDR").unwrap_or(DEFAULT_ADDRESS);
    info!("Starting server on {}:{}", address, port);
    server::prepare_server(blockchain, address, port)
        .expect("Error while creating the server")
        .launch();
}
