use std::io::Cursor;

use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::{Request, Response};

#[derive(Debug, Fail)]
pub enum BlockchainError {
    // #[fail(display = "Key pair {} already exists", path)]
    // KeyPairAlreadyExists { path: String },
    #[fail(display = "Invalid block")]
    InvalidBlock,
    #[fail(display = "Cannot get lock")]
    CannotGetLock,
    #[fail(display = "Empty chain")]
    EmptyChain,
}

impl Responder<'static> for BlockchainError {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        use BlockchainError::*;
        let msg = format!("{}", self);
        let status = match self {
            InvalidBlock => Status::NotAcceptable,
            EmptyChain => Status::Conflict,
            _ => Status::InternalServerError,
        };
        Response::build()
            .header(ContentType::Plain)
            .sized_body(Cursor::new(msg))
            .status(status)
            .ok()
    }
}
