/// Version number if build using cargo (is set and evaluated at compile time).
const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

/// Builds the cli argument parser and parses the arguments.
pub fn build_cli() -> ::clap::ArgMatches<'static> {
    clap_app!(blockchain =>
              (version: VERSION.unwrap_or("unknown version")) // if not build using cargo
              (author: "Valentin Brandl <mail@vbrandl.net>")
              (about: "PoC blockchain")
              (@arg BLOCKCHAIN: -b --blockchain +takes_value "Path to the persisted blockchain")
              (@arg PORT: -p --port +takes_value "Port to listen on (Defaults to 1337)")
              (@arg ADDR: -a --address +takes_value "Address to listen on (Defaults to localhost)")
             ).get_matches()
}
