extern crate argon2rs;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate failure;

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

extern crate data;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod cli;
mod key;

fn main() {
    env_logger::init();
    let matches = cli::build_cli();

    // let key = key::get_key().expect("Cannot get key");

    // generate keypair and exit
    if let Some(matches) = matches.subcommand_matches("generate_keypair") {
        let path = matches.value_of("PATH").unwrap_or(key::DEFAULT_KEY_PATH);
        let pwd = key::get_password().expect("Cannot read password");
        let key_pair = key::EncryptedKeyPair::new(&pwd).expect("Cannot generate key");
        let write = key_pair.write_to_file(path);
        if write.is_ok() {
            info!("Keypair has been created");
            ::std::process::exit(0);
        } else {
            error!("Failure when creating the keypair");
            ::std::process::exit(1);
        }
    }

    let key_path = matches.value_of("KEYPAIR").unwrap_or(key::DEFAULT_KEY_PATH);
    let pwd = key::get_password().expect("Cannot read password");
    let key_pair = key::KeyPair::from_file(key_path, &pwd).expect("Cannot read keypair");
    info!("Loading keypair from {}", key_path);

    // let key_pair = key::read_keypair(key_path).expect("Invalid key data");
}
