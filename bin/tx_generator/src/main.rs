#![deny(warnings, missing_docs)]
//! This crate provides functionality to generate new usage transactions and append them to the
//! blockchain.

#[macro_use]
extern crate clap;

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate client;
extern crate cryptography;
extern crate data;

mod cli;

use std::io::{BufWriter, Write};
use std::fs::OpenOptions;

use data::Block;

fn main() {
    env_logger::init();
    let matches = cli::build_cli();

    if let Some(matches) = matches.subcommand_matches("generate_keypair") {
        // generate keypair
        let path = matches
            .value_of("PATH")
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
    } else if let Some(matches) = matches.subcommand_matches("generate_transaction") {
        let url = matches.value_of("HOST").unwrap();
        let key_path = matches
            .value_of("KEYPAIR")
            .unwrap_or(cryptography::DEFAULT_KEY_PATH);
        let pwd = cryptography::get_password().expect("Cannot read password");
        let key_pair =
            cryptography::KeyPair::from_file(key_path, &pwd).expect("Cannot read keypair");
        let usage: u64 = matches
            .value_of("USAGE")
            .unwrap()
            .parse()
            .expect("Cannot parse usage");
        info!("Loading key pair from {}", key_path);
        let client = client::Client::new(url).expect("Invalid url");

        info!("Generating data");
        let tx = data::tx::Data::Usage(usage);
        info!("Signing data");
        let signed_data =
            cryptography::sign_data(&key_pair, tx).expect("Error while signing the data");
        info!("Receiving latest block");
        let latest = client
            .latest_block()
            .expect(&format!("Can't get latest block from {}", url));
        info!("Generating new block");
        let block: Block =
            data::block::Block::new_with_hash(signed_data, latest.hash(), data::DIFFICULTY);
        info!("Performing proof of work");
        let block = block.proof_of_work();
        client
            .append(&block)
            .expect("Error while appending the block");
        info!("New block has been appended to the blockchain");
    } else if let Some(matches) = matches.subcommand_matches("export_public_key") {
        let key_path = matches
            .value_of("KEYPAIR")
            .unwrap_or(cryptography::DEFAULT_KEY_PATH);
        let out_path = matches.value_of("PATH").unwrap();
        let pwd = cryptography::get_password().expect("Cannot read password");
        info!("Loading keypair from {}", key_path);
        let key_pair =
            cryptography::KeyPair::from_file(key_path, &pwd).expect("Cannot read keypair");
        let pub_key = key_pair.public_key_bytes();
        info!("Creating and opening outfile {}", out_path);
        let mut writer = BufWriter::new(
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(out_path)
                .expect(&format!("Unable to create new file {}", out_path)),
        );
        info!("Writing public key to outfile {}", out_path);
        writer
            .write_all(pub_key.bytes())
            .expect(&format!("Unable to write to file {}", out_path));
        info!("Public key successfully exported to {}", out_path);
    }
}
