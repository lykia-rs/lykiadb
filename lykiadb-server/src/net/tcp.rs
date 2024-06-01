use bytes::BytesMut;
use tokio::io::{BufWriter, AsyncReadExt, AsyncWriteExt, copy};
use tokio::net::TcpStream;
use crate::net::{CommunicationError, Message};

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
