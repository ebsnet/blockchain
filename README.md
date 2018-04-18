# Blockchain


## Repository Structure

### `lib`

Libraries that offer shared utility methods.

#### `lib/blockchain`

Generic PoW blockchain library

#### `lib/client`

HTTP client library to communicate with the web service

#### `lib/cryptography`

Cryptography functions for handling key pairs, secure memory management, signing and verifying data

#### `lib/data`

Exposes a concrete blockchain,


## Building The Components

This project was developed using the Rust nightly compiler from `2018-02-14`. Newer versions should work as well. To
install this specific version run

```
rustup install nightly-2018-02-14
```

Alternatively everything can be build inside a Docker container using the `docker_build_musl.sh` and
`docker_build_glibc.sh` scripts.


## Logging

Log levels are handled via the `RUST_LOG` environment variable. The default log level is `error`. To view `info` logs
(which is recommended), the environment variable needs to be set. It has the following structure:

```
RUST_LOG="<component>=<level>"
RUST_LOG="<component1>=<level>,<component2>=<level>"
```

Component being the name of a crate. The environment variable can either be set globally for the current shell
session using `export RUST_LOG="webservice=info"` or by starting a component like this: `RUST_LOG="webservice=info
./webservice"`

The following line will set the log level for all components of this repository in the current shell session:

```
export RUST_LOG="webservice=info,tx_generator=info,invoice_generator=info"
```


## Usage example

1. Start the web service: (this will start a web server listening on `localhost:1337`, persisting the blockchain to
   `./blockchain.dat`; for more information view the help dialog by passing the `--help` flag)
  ```
  RUST_LOG="webservice=info" ./webservice
  ```

1. Generate a key pair for the user:
  ```
  RUST_LOG="tx_generator=info" ./tx_generator generate_keypair -p user.key
  ```

1. Export the user's public key:
  ```
  RUST_LOG="tx_generator=info" ./tx_generator export_public_key -k user.key user.pub
  ```

1. Generate a key pair for the billing party:
  ```
  RUST_LOG="invoice_generator=info" ./invoice_generator -k billing.key generate_keypair
  ```

1. Initialize billing for the user:
  ```
  RUST_LOG="invoice_generator=info" ./invoice_generator -k billing.key initialize_billing -h http://localhost:1337/ --publickey user.pub
  ```

1. Generate usage transactions:
  ```
  RUST_LOG="tx_generator=info" ./tx_generator generate_transaction -h http://localhost:1337 <usage> -k user.key
  ```

1. Generate invoice for user:
  ```
  RUST_LOG="invoice_generator=info" ./invoice_generator -k billing.key create_invoice --publickey user.pub -h http://localhost:1337/
  ```
