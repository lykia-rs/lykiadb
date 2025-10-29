pub mod tcp;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Span, StandardError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Run(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardErrorImpl {
    pub message: String,
    pub hint: String,
    pub error_code: String,
    pub span: Option<Span>,
}

impl StandardErrorImpl {
    pub fn new(
        message: &str,
        hint: &str,
        span: Option<Span>,
    ) -> Self {
        StandardErrorImpl {
            message: message.to_owned(),
            hint: hint.to_owned(),
            error_code: "000".to_owned(),
            span: span.clone(),
        }
    }
}

impl StandardError for StandardErrorImpl {
    fn get_message(&self) -> String {
        self.message.clone()
    }
    fn get_hint(&self) -> String {
        self.hint.clone()
    }
    fn get_error_code(&self) -> String {
        self.error_code.clone()
    }
    fn get_span(&self) -> Option<Span> {
        self.span.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Value(Value),
    Program(Value),
    Error(StandardErrorImpl),
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
