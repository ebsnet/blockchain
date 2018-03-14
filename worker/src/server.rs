use rocket::State;
use rocket::config::{Config, Environment};
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::Json;

use failure::Error;

use error::BlockchainError;
use state::ServerState;
use data::Block;

#[get("/")]
fn index() -> &'static str {
    "index"
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

// #[post("/user_tx", format = "application/json", data = "<key>")]
// fn get_tx_from_user(
//     state: State<ServerState>,
//     key: Key,
// ) -> Result<Json<Vec<Block>>, BlockchainError> {
//     unimplemented!()
// }

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
        .mount("/", routes![index, latest_block, append])
        .manage(state))
}
