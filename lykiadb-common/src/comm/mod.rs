pub mod client;
pub mod tcp;

use bson::Bson;
use serde::{Deserialize, Serialize};

use crate::error::InputError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Run(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Value(Bson, u64),
    Error(InputError, u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Request(Request),
    Response(Response),
}

#[derive(Debug)]
pub enum CommunicationError {
    BsonError(bson::error::Error),
    IoError(std::io::Error),
    GenericError(String),
}

impl From<std::io::Error> for CommunicationError {
    fn from(value: std::io::Error) -> Self {
        CommunicationError::IoError(value)
    }
}

impl From<bson::error::Error> for CommunicationError {
    fn from(value: bson::error::Error) -> Self {
        CommunicationError::BsonError(value)
    }
}
