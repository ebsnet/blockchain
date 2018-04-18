use rocket::State;
use rocket::config::{Config, Environment};
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::Json;

use failure::Error;

use error::BlockchainError;
use state::ServerState;
use data::{Block, Blockchain};
use cryptography::BillingQuery;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[get("/")]
fn index() -> String {
    format!(
        r#"
    Blockchain webservice v{}

    The following operations are supported:

    POST /append

        Appends a new block (passed als "application/json")

    GET /latest_block

        Returns the latest block as a JSON string

    POST /since_last_billing

        Returns the part of the blockchain since the last billing for a specified user
            "#,
        VERSION.unwrap_or("unknown")
    )
}

#[get("/latest_block")]
fn latest_block(state: State<ServerState>) -> Result<Json<Block>, BlockchainError> {
    state.latest_block().map(Json)
}

#[post("/append", format = "application/json", data = "<block>")]
fn append(
    state: State<ServerState>,
    block: Json<Block>,
) -> Result<status::Custom<&'static str>, BlockchainError> {
    let path = state.path();
    state
        .append(block.0, path)
        .map(|_| status::Custom(Status::Accepted, "block was appended"))
}

#[post("/since_last_billing", format = "application/json", data = "<query>")]
fn since_last_billing(
    state: State<ServerState>,
    query: Json<BillingQuery>,
) -> Result<Option<Json<Blockchain>>, BlockchainError> {
    state.last_billing(&query.0).map(|opt| opt.map(Json))
}

pub fn prepare_server(
    state: ServerState,
    address: &str,
    port: u16,
) -> Result<::rocket::Rocket, Error> {
    let config = Config::build(Environment::Staging)
        .address(address)
        .port(port)
        .finalize()?;

    Ok(::rocket::custom(config, true)
        .mount(
            "/",
            routes![index, latest_block, append, since_last_billing],
        )
        .manage(state))
}
