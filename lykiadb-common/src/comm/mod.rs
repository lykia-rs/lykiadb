pub mod client;
pub mod tcp;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::InputError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Run(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Value(Value, u128),
    Program(Value, u128),
    Error(InputError, u128),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Request(Request),
    Response(Response),
}

#[derive(Debug)]
pub enum CommunicationError {
    BsonError(bson::ser::Error),
    IoError(std::io::Error),
    GenericError(String),
}

impl From<std::io::Error> for CommunicationError {
    fn from(value: std::io::Error) -> Self {
        CommunicationError::IoError(value)
    }
}

impl From<bson::ser::Error> for CommunicationError {
    fn from(value: bson::ser::Error) -> Self {
        CommunicationError::BsonError(value)
    }
}
