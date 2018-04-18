#![deny(warnings, missing_docs)]
//! This crate generates invoices for a user.

extern crate chrono;
#[macro_use]
extern crate clap;

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate client;
extern crate cryptography;
extern crate data;

mod cli;
mod invoice;
mod ask;

use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

use invoice::{Invoice, InvoicePosition};

use data::Block;
use data::tx::Data;

use cryptography::{validate_signature, BillingQuery};

fn create_invoice(matches: &clap::ArgMatches<'static>) {
    let key_pair = matches
        .value_of("KEYPAIR")
        .unwrap_or(cryptography::DEFAULT_KEY_PATH);
    let pwd = cryptography::get_password().expect("Cannot read password");
    let pub_key = matches.value_of("PUBKEY").unwrap();
    info!("Loading key pair from {}", key_pair);
    let key_pair = cryptography::KeyPair::from_file(key_pair, &pwd).expect("Cannot read key pair");
    info!("Loading public key from {}", pub_key);
    let pub_key = cryptography::PublicKey::load_from_file(pub_key).expect("Cannot load public key");
    let url = matches.value_of("HOST").unwrap();
    info!("Receiving latest billing operation");
    let client = client::Client::new(url).expect("Invalid host");
    let query = BillingQuery::new(key_pair.public_key_bytes(), pub_key.fingerprint());
    let result = client
        .since_last_billing(&query)
        .expect("Error requesting the latest billing");
    if let Some(chain) = result {
        info!("Received subchain, calculating invoice...");
        let positions = chain
            .iter()
            .filter(|blk| match *blk.data().data() {
                Data::Usage(_) => true,
                _ => false,
            })
            .filter(|blk| validate_signature(&pub_key, blk.data()).unwrap_or(false))
            .map(|blk| {
                InvoicePosition::new(
                    blk.time(),
                    match *blk.data().data() {
                        Data::Usage(usg) => usg,
                        _ => unreachable!(),
                    },
                )
            })
            .collect::<BTreeSet<_>>();
        if positions.is_empty() {
            error!("No new usage transactions since the last billing operation. Exiting...");
            std::process::exit(0);
        }
        let invoice = Invoice::new(pub_key.clone(), positions);
        let out_file = format!("{}_{}.txt", invoice.user(), timestamp());
        info!("Writing invoice to file: {}", out_file);
        invoice
            .write_to_file(out_file)
            .expect("Could not write invoice to file");
        println!("{}", invoice);
        if ask::ask("Write billing to blockchain") {
            info!("Creating billing block");
            let data = pub_key.to_billing();
            info!("Signing data");
            let signed_data =
                cryptography::sign_data(&key_pair, data).expect("Error while signing the data");
            info!("Receiving latest block");
            let client = client::Client::new(url).expect("Invalid host");
            let latest = client.latest_block().map(|b| b.hash()).unwrap_or_default();
            // .expect(&format!("Can't get latest block from {}", url));
            info!("Generating new block");
            let block: Block =
                data::block::Block::new_with_hash(signed_data, latest, data::DIFFICULTY);
            info!("Performing proof of work");
            let block = block.proof_of_work();
            client
                .append(&block)
                .expect("Error while appending the block");
            info!("New block has been appended to the blockchain");
        }
    } else {
        error!("Billing not initialized for supplied public key");
        std::process::exit(1);
    }
}

fn generate_keypair(matches: &clap::ArgMatches<'static>) {
    // generate keypair
    let path = matches
        .value_of("KEYPAIR")
        .unwrap_or(cryptography::DEFAULT_KEY_PATH);
    let pwd = cryptography::get_password().expect("Cannot read password");
    info!("Generating key pair");
    let key_pair = cryptography::EncryptedKeyPair::new(&pwd).expect("Cannot generate key");
    info!("Writing key pair to file");
    key_pair
        .write_to_file(path)
        .map(|w| {
            info!("Key pair has been created");
            w
        })
        .expect("Failure when creating the keypair");
    info!("Key pair has been generated");
}

fn initialize_billing(matches: &clap::ArgMatches<'static>) {
    let key_pair = matches
        .value_of("KEYPAIR")
        .unwrap_or(cryptography::DEFAULT_KEY_PATH);
    let pwd = cryptography::get_password().expect("Cannot read password");
    let pub_key = matches.value_of("PUBKEY").unwrap();
    info!("Loading key pair from {}", key_pair);
    let key_pair = cryptography::KeyPair::from_file(key_pair, &pwd).expect("Cannot read key pair");
    info!("Loading public key from {}", pub_key);
    let pub_key = cryptography::PublicKey::load_from_file(pub_key).expect("Cannot load public key");
    let url = matches.value_of("HOST").unwrap();
    info!("Generating data");
    let data = pub_key.to_billing();
    info!("Signing data");
    let signed_data =
        cryptography::sign_data(&key_pair, data).expect("Error while signing the data");
    info!("Receiving latest block");
    let client = client::Client::new(url).expect("Invalid host");
    let latest = client.latest_block().map(|b| b.hash()).unwrap_or_default();
    // .expect(&format!("Can't get latest block from {}", url));
    info!("Generating new block");
    let block: Block = data::block::Block::new_with_hash(signed_data, latest, data::DIFFICULTY);
    info!("Performing proof of work");
    let block = block.proof_of_work();
    client
        .append(&block)
        .expect("Error while appending the block");
    info!("New block has been appended to the blockchain");
}

fn main() {
    env_logger::init();
    let matches = cli::build_cli();

    if let Some(matches) = matches.subcommand_matches("generate_keypair") {
        generate_keypair(matches);
    } else if let Some(matches) = matches.subcommand_matches("initialize_billing") {
        initialize_billing(matches);
    } else if let Some(matches) = matches.subcommand_matches("create_invoice") {
        create_invoice(matches);
    }
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error while getting timestamp")
        .as_secs()
}
