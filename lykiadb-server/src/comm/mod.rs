use crate::engine::interpreter::Interpreter;
use crate::engine::{Runtime, RuntimeMode};
use crate::value::RV;
use ::std::time::Instant;
use tcp::TcpConnection;
use tokio::net::TcpStream;
use tracing::{error, info};

pub mod tcp;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::error::ExecutionError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Run(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Value(RV),
    Program(Value),
    Error(ExecutionError),
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

pub struct ServerSession {
    conn: TcpConnection,
    runtime: Runtime,
}

impl ServerSession {
    pub fn new(stream: TcpStream) -> Self {
        ServerSession {
            conn: TcpConnection::new(stream),
            runtime: Runtime::new(RuntimeMode::File, Interpreter::new(None, true)),
        }
    }

    pub async fn handle(&mut self) {
        while let Some(message) = self.conn.read().await.unwrap() {
            // Here we measure the time it takes to process a message
            let start = Instant::now();
            match &message {
                Message::Request(req) => match req {
                    Request::Run(command) => {
                        let execution = self.runtime.interpret(command);
                        
                        let response = if execution.is_ok() {
                            Response::Value(execution.ok().unwrap())
                        } else {
                            Response::Error(execution.err().unwrap())
                        };

                        self.conn.write(Message::Response(response)).await.unwrap();
                    }
                },
                _ => error!("Unsupported message type"),
            }
            let elapsed = start.elapsed();
            info!("{:?} (took {:?})", message, elapsed);
        }
    }

    pub async fn send(&mut self, msg: Message) -> Result<(), CommunicationError> {
        self.conn.write(msg).await
    }
}
