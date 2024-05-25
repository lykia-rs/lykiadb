use lykiadb_server::net::{CommunicationError, Connection, Message};
use tokio::net::TcpStream;
use crate::session::ClientSession;

pub(crate) struct TcpClientSession {
    conn: Connection,
}

impl TcpClientSession {
    pub fn new(stream: TcpStream) -> Self {
        TcpClientSession {
            conn: Connection::new(stream),
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
