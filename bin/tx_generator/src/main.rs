extern crate argon2rs;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate failure;

extern crate openssl;

extern crate rand;

extern crate ring;
extern crate untrusted;

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate rpassword;

extern crate crypto;

extern crate memsec;
extern crate seckey;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate client;
extern crate data;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod cli;
mod key;

use data::Block;

fn generate_data() -> data::tx::Data {
    unimplemented!()
}

fn main() {
    env_logger::init();
    let matches = cli::build_cli();

    // let key = key::get_key().expect("Cannot get key");

    if let Some(matches) = matches.subcommand_matches("generate_keypair") {
        // generate keypair
        let path = matches.value_of("PATH").unwrap_or(key::DEFAULT_KEY_PATH);
        let pwd = key::get_password().expect("Cannot read password");
        let key_pair = key::EncryptedKeyPair::new(&pwd).expect("Cannot generate key");
        key_pair
            .write_to_file(path)
            .map(|w| {
                info!("Keypair has been created");
                w
            })
            .expect("Failure when creating the keypair");
    } else if let Some(matches) = matches.subcommand_matches("generate_transaction") {
        let url = matches.value_of("HOST").unwrap();
        let key_path = matches.value_of("KEYPAIR").unwrap_or(key::DEFAULT_KEY_PATH);
        let pwd = key::get_password().expect("Cannot read password");
        let key_pair = key::KeyPair::from_file(key_path, &pwd).expect("Cannot read keypair");
        info!("Loading keypair from {}", key_path);
        let client = client::Client::new(url);

        info!("Generating data");
        let tx = generate_data();
        info!("Signing data");
        let signed_data = key::sign_data(&key_pair, tx).expect("Error while signing the data");
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
    }
}
