use lykiadb_server::net::{CommunicationError, Connection, Message};
use tokio::net::TcpStream;

pub struct ClientSession {
    conn: Connection,
}

impl ClientSession {
    pub fn new(stream: TcpStream) -> Self {
        ClientSession {
            conn: Connection::new(stream),
        }
    }

    pub async fn handle(&mut self) -> Result<Message, ()> {
        match self.conn.read().await.unwrap() {
            Some(message) => Ok(message),
            None => Err(())
        }
    }

    pub async fn send(&mut self, msg: Message) -> Result<(), CommunicationError> {
        self.conn.write(msg).await
    }
}
