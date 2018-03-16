/// Version number if build using cargo (is set and evaluated at compile time).
const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

/// Builds the cli argument parser and parses the arguments.
pub fn build_cli() -> ::clap::ArgMatches<'static> {
    clap_app!(tx_generator =>
              (version: VERSION.unwrap_or("unknown version")) // if not build using cargo
              (author: "Valentin Brandl <mail@vbrandl.net>")
              (about: "Transaction generator")
              (@subcommand generate_keypair =>
               (about: "Generates a new keypair")
               (version: "1.0")
               (@arg PATH: -p --path +takes_value "Path to write the keypair to (Defaults to ./default.key)")
              )
              (@subcommand generate_transaction =>
               (about: "Generates a new transaction, mines a block and appends it to the blockchain")
               (version: "1.0")
               (@arg KEYPAIR: -k --keypair +takes_value "Path to the keypair (Defaults to ./default.key)")
               (@arg HOST: -h --host +takes_value +required "URL of the webservice")
              )
             ).get_matches()
}
