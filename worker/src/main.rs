#[macro_use]
extern crate clap;

#[macro_use]
extern crate failure;

extern crate ring;
extern crate untrusted;

extern crate env_logger;
#[macro_use]
extern crate log;

mod tx;

use failure::Error;

use ring::signature::Ed25519KeyPair;
use ring::rand;

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const DEFAULT_KEY_PATH: &'static str = "./default.key";
const DEFAULT_PORT: &'static str = "1337";
const DEFAULT_BC_PATH: &'static str = "./blockchain.dat";
const KEY_SIZE: usize = 85;

#[derive(Debug, Fail)]
enum BlockchainError {
    #[fail(display = "keypair {} already exists", path)]
    KeyPairAlreadyExists { path: String },
}

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
    let matches = clap_app!(blockchain =>
                            (version: VERSION.unwrap_or("unknown"))
                            (author: "Valentin Brandl <mail@vbrandl.net>")
                            (about: "PoC blockchain")
                            (@arg KEYPAIR: -k --keypair +takes_value "Path to the keypair (Defaults to ./default.key)")
                            (@arg PORT: -p --port +takes_value "The HTTP port to listen on (Defaults to 1337)")
                            (@arg BLOCKCHAIN: -b --blockchain +takes_value "Path to the persisted blockchain")
                            (@subcommand generate_keypair =>
                             (about: "Generates a new keypair")
                             (version: "1.0")
                             (@arg PATH: -p --path +takes_value "Path to write the keypair to (Defaults to ./default.key)")
                             )
                            ).get_matches();

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

    let port: u16 = matches
        .value_of("PORT") // load port from args
        .unwrap_or(DEFAULT_PORT) // or fallback to default
        .parse() // parse into u16
        .expect("Cannot parse port"); // or fail
    debug!("Using port {}", port);

    let key_path = matches.value_of("KEYPAIR").unwrap_or(DEFAULT_KEY_PATH);
    info!("Loading keypair from {}", key_path);

    let key_pair = read_keypair(key_path).expect("Invalid key data");
}
