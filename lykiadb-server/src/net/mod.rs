use bytes::BytesMut;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
    io::{copy, AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::runtime::{error::ExecutionError, types::RV};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Run(String),
    Ast(String),
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

pub struct TcpConnection {
    pub stream: BufWriter<TcpStream>,
    pub read_buffer: BytesMut,
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> TcpConnection {
        TcpConnection {
            stream: BufWriter::new(stream),
            read_buffer: BytesMut::with_capacity(4096),
        }
    }

    pub async fn read(&mut self) -> Result<Option<Message>, CommunicationError> {
        loop {
            // TODO(vck): Replace .to_vec call with something cheaper
            if let Ok(parsed) = bson::from_slice::<Message>(&self.read_buffer.to_vec()) {
                self.read_buffer.clear();
                return Ok(Some(parsed));
            }

            if 0 == self.stream.read_buf(&mut self.read_buffer).await? {
                if self.read_buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(CommunicationError::GenericError(
                        "Connection reset by peer".to_owned(),
                    ));
                }
            }
        }
    }

    pub async fn write(&mut self, message: Message) -> Result<(), CommunicationError> {
        let vec = bson::to_vec(&bson::to_bson(&message)?)?;
        let mut buffer = vec.as_slice();
        copy(&mut buffer, &mut self.stream).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
