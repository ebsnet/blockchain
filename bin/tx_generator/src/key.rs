use openssl::symm;

use failure::Error;

use rand::Rng;

use ring::signature::{Ed25519KeyPair, Signature};
use ring::rand;

use seckey::{zero, SecKey, SecReadGuard};
use memsec::memzero;

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Default path for the key pair.
pub const DEFAULT_KEY_PATH: &str = "./default.key";
/// Size of the salt.
const SALT_SIZE: usize = 32;
/// Size of the nonce.
const NONCE_SIZE: usize = 16;
/// Name of the environment variable where the password might be stored.
const PWD_ENV: &str = "PRIVATE_KEY_PASS";

#[derive(Debug, Fail)]
pub enum KeyError {
    #[fail(display = "Cannot create secure memory")]
    SecureMemoryError,
    #[fail(display = "Cannot read key")]
    ReadKeyError,
    #[fail(display = "Cannot generate key pair")]
    GenerateKeyError,
}

///  An encrypted key pair, holding the encrypted data, the nonce used to decrypt the data and the
///  salt used to derive the encryption key.
#[derive(Serialize, Deserialize)]
pub struct EncryptedKeyPair {
    salt: [char; SALT_SIZE],
    nonce: [u8; NONCE_SIZE],
    key: Vec<u8>,
}

impl EncryptedKeyPair {
    /// Creates a new key pair and encrypts it using the password and a randomly generated nonce
    /// and salt.
    pub fn new(pwd: &Password) -> Result<Self, Error> {
        let salt = random_salt();
        let nonce = random_nonce();
        let enc_key = EncryptionKey::new(pwd, &salt.iter().collect::<String>())?;
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)?;
        let enc_bytes = encrypt(&pkcs8_bytes, &nonce, &enc_key)?
            .iter()
            .cloned()
            .collect::<Vec<u8>>();
        Ok(Self {
            salt: salt,
            nonce: nonce,
            key: enc_bytes,
        })
    }

    /// Writes an JSON encoded, encrypted key pair to a file.
    pub fn write_to_file<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let json = ::serde_json::to_string(self)?;
        let mut writer =
            BufWriter::new(OpenOptions::new().create_new(true).write(true).open(path)?);
        writer.write_all(json.as_bytes())?;
        Ok(())
    }
}

/// Wrapper that holds a password in a secure memory.
pub struct Password(SecKey<String>);

impl Password {
    /// Creates a new instance of password. If securing the memory area fails, the string gets
    /// overwritten with `0x00`.
    pub fn new(pw: String) -> Result<Password, KeyError> {
        Ok(Password(SecKey::new(pw).map_err(|mut val| {
            zero(val.as_mut_str());
            KeyError::SecureMemoryError // and return error
        })?))
    }

    /// Returns read guard to the encapsulated password.
    fn read(&self) -> SecReadGuard<String> {
        self.0.read()
    }
}

/// Wrapper that holds an Ed25519 key pair in a secure memory area.
pub struct KeyPair(SecKey<Ed25519KeyPair>);

impl KeyPair {
    /// Loads a key pair from a file using the provided password.
    pub fn from_file<P>(path: P, pwd: &Password) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let content = read_file_to_string(path)?;
        let enc_key_pair: EncryptedKeyPair = ::serde_json::from_str(&content)?;
        let encryption_key =
            EncryptionKey::new(pwd, &enc_key_pair.salt.iter().collect::<String>())?;
        let dec = decrypt(&enc_key_pair.key, &enc_key_pair.nonce, &encryption_key)?;
        let pair = KeyPair(
            SecKey::new(Ed25519KeyPair::from_pkcs8(::untrusted::Input::from(
                &*dec.read(),
            ))?).map_err(|mut val| {
                custom_zero(&mut val);
                KeyError::SecureMemoryError // and return error
            })?,
        );
        Ok(pair)
    }
}

/// Wrapper that holds an encryption key in a secure memory area.
struct EncryptionKey(SecKey<[u8; 32]>);

impl EncryptionKey {
    /// Derives an encryption key from a password and salt using the argon2i derivation function.
    fn new(pass: &Password, salt: &str) -> Result<Self, KeyError> {
        let p = pass.read();
        let key = ::argon2rs::argon2i_simple(&p, salt);
        Ok(EncryptionKey(SecKey::new(key).map_err(|mut val| {
            // store in secret memory
            zero(&mut val); // or zero out the secret
            KeyError::SecureMemoryError // and return error
        })?))
    }

    #[cfg(test)]
    /// Wraps a byte array in a secure memory area.
    fn from_bytes(bytes: [u8; 32]) -> Result<Self, KeyError> {
        Ok(EncryptionKey(SecKey::new(bytes).map_err(|mut val| {
            // store in secret memory
            zero(&mut val); // or zero out the secret
            KeyError::SecureMemoryError // and return error
        })?))
    }
}

/// Gets a password by first checking the system environment for `PRIVATE_KEY_PASS`, if the
/// variable does not exist, the user is prompted to enter the password.
pub fn get_password() -> Result<Password, KeyError> {
    let key = env::var(PWD_ENV) // read from environment
        .or_else(|_| ::rpassword::prompt_password_stderr("Enter password: ")) // or prompt user
        .map(|s| s.to_owned())
        .map_err(|_| KeyError::ReadKeyError)?; // or fail
    Password::new(key)
}

/// Signs data using a `KeyPair`
pub fn sign_data(key: &KeyPair, data: &[u8]) -> Signature {
    let key = key.0.read();
    key.sign(data)
}

/// Read a file into a string.
fn read_file_to_string<P>(path: P) -> Result<String, Error>
where
    P: AsRef<Path>,
{
    let mut reader = BufReader::new(File::open(path)?);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    Ok(content)
}

/// Decrypt data into a secret memory area using a supplied nonce and encryption key using the
/// `AES` algorithm.
fn decrypt(data: &[u8], nonce: &[u8], key: &EncryptionKey) -> Result<SecKey<Vec<u8>>, Error> {
    let dec = symm::decrypt(
        symm::Cipher::aes_256_cbc(),
        &*key.0.read(),
        Some(nonce),
        data,
    )?;
    Ok(SecKey::new(dec).map_err(|mut val| {
        custom_zero(&mut val);
        KeyError::SecureMemoryError
    })?)
}

/// Encrypts data using a supplied nonce and encryption key using the `AES` algorithm.
fn encrypt(data: &[u8], nonce: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, Error> {
    let enc = symm::encrypt(
        symm::Cipher::aes_256_cbc(),
        &*key.0.read(),
        Some(nonce),
        data,
    )?;
    Ok(enc)
}

/// Generates a random salt.
fn random_salt() -> [char; SALT_SIZE] {
    let mut rng = ::rand::thread_rng();
    rng.gen::<[char; SALT_SIZE]>()
}

/// Generates a random nonce.
fn random_nonce() -> [u8; NONCE_SIZE] {
    let mut rng = ::rand::thread_rng();
    rng.gen::<[u8; NONCE_SIZE]>()
}

/// Overwrites a supplied memory area with `0x00`
fn custom_zero<T: Sized>(t: &mut T) {
    unsafe { memzero(t as *mut T as *mut u8, ::std::mem::size_of_val(t)) };
}

#[cfg(test)]
mod test {
    use super::*;

    quickcheck! {
        /// Encrypting and decrypting any data should result in the original input.
        fn encrypt_and_decrypt_is_identity(data: Vec<u8>) -> bool {
            let mut rng = ::rand::thread_rng();
            let bytes = rng.gen::<[u8; 32]>(); // generate a random key
            let sec_key = EncryptionKey::from_bytes(bytes).unwrap();
            let nonce = random_nonce(); // generate a random nonce
            let enc = encrypt(&data, &nonce, &sec_key).unwrap();
            let dec = decrypt(&enc, &nonce, &sec_key).unwrap(); // decrypt the data
            let dec = &*dec.read();
            dec == &data // compare
        }
    }
}
