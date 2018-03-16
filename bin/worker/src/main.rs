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

extern crate data;

mod cli;
mod error;
mod server;
mod state;
mod wrapper;

use error::BlockchainError;
use state::ServerState;

use data::blockchain;

use failure::Error;

use ring::signature::Ed25519KeyPair;
use ring::rand;

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

const DEFAULT_KEY_PATH: &str = "./default.key";
/// Default path to look for the blockchain.
const DEFAULT_BC_PATH: &str = "./blockchain.dat";
/// Default path for the webserver to listen on.
const DEFAULT_PORT: &str = "1337";
/// Default address for the webserver to listen on.
const DEFAULT_ADDRESS: &str = "localhost";
/// Size of Ed25519 keys.
const KEY_SIZE: usize = 85;

fn read_keypair<P>(path: P) -> Result<Ed25519KeyPair, Error>
where
    P: AsRef<Path>,
{
    let mut file = BufReader::with_capacity(KEY_SIZE, File::open(path)?);
    let mut pkcs8_bytes = Vec::with_capacity(KEY_SIZE);
    file.read_to_end(&mut pkcs8_bytes)?;
    let key_pair = Ed25519KeyPair::from_pkcs8(untrusted::Input::from(&pkcs8_bytes))?;
    Ok(key_pair)
}

fn generate_keypair<P>(path: P) -> Result<Ed25519KeyPair, Error>
where
    P: AsRef<Path>,
{
    let rng = rand::SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)?;
    let mut file = BufWriter::new(OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.as_ref())
        .map_err(|_| BlockchainError::KeyPairAlreadyExists {
            path: format!("{:?}", path.as_ref()),
        })?);
    file.write_all(&pkcs8_bytes)?;
    let key_pair = Ed25519KeyPair::from_pkcs8(untrusted::Input::from(&pkcs8_bytes))?;
    Ok(key_pair)
}

fn main() {
    env_logger::init();
    let matches = cli::build_cli();

    // generate keypair and exit
    if let Some(matches) = matches.subcommand_matches("generate_keypair") {
        let path = matches.value_of("PATH").unwrap_or(DEFAULT_KEY_PATH);
        let key_pair = generate_keypair(path);
        if let Some(e) = key_pair.err() {
            error!("Failure when creating the keypair: {}", e);
            ::std::process::exit(1);
        } else {
            info!("Keypair has been created");
            ::std::process::exit(0);
        }
    }

    let key_path = matches.value_of("KEYPAIR").unwrap_or(DEFAULT_KEY_PATH);
    info!("Loading keypair from {}", key_path);

    let key_pair = read_keypair(key_path).expect("Invalid key data");

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
    server::prepare_server(blockchain, address, port)
        .expect("Error while creating the server")
        .launch();
}
