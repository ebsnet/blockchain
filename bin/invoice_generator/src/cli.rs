/// Version number if build using cargo (is set and evaluated at compile time).
const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

/// Builds the cli argument parser and parses the arguments.
pub fn build_cli() -> ::clap::ArgMatches<'static> {
    clap_app!(invoice_generator =>
              (version: VERSION.unwrap_or("unknown version")) // if not build using cargo
              (author: "Valentin Brandl <mail@vbrandl.net>")
              (about: "Invoice generator")
              (@arg KEYPAIR: -k --keypair +takes_value "Path to the key pair (Defaults to ./default.key)")
              (@subcommand generate_keypair =>
               (about: "Generates a new key pair")
               (version: "1.0")
              )
              (@subcommand initialize_billing =>
               (about: "Initialize the billing process associated with a public key")
               (version: VERSION.unwrap_or("unknown version"))
               (@arg PUBKEY: -p --publickey +takes_value +required "Public key to initialize the billing process for")
               (@arg HOST: -h --host +takes_value +required "URL of the webservice")
              )
              (@subcommand create_invoice =>
               (about: "Create an invoice for a public key")
               (version: VERSION.unwrap_or("unknown version"))
               (@arg PUBKEY: -p --publickey +takes_value +required "Public key to initialize the billing process for")
               (@arg HOST: -h --host +takes_value +required "URL of the webservice")
              )
             ).get_matches()
}
