use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use lykiadb_server::net::{CommunicationError, Message, TcpConnection};
use crate::session::ClientSession;

pub(crate) struct TcpClientSession {
    conn: TcpConnection,
}

impl TcpClientSession {
    pub fn new(stream: TcpStream) -> Self {
        TcpClientSession {
            conn: TcpConnection::new(stream),
        }
    }

    async fn handle(&mut self) -> Result<Message, ()> {
        match self.conn.read().await.unwrap() {
            Some(message) => Ok(message),
            None => Err(()),
        }
    }

    async fn send(&mut self, msg: Message) -> Result<(), CommunicationError> {
        self.conn.write(msg).await
    }
}
impl ClientSession for TcpClientSession {
    async fn send_receive(&mut self, msg: Message) -> Result<Message, ()> {
        self.send(msg).await.unwrap();
        self.handle().await
    }
}
