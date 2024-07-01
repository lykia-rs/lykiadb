use crate::session::ClientSession;
use lykiadb_server::net::tcp::TcpConnection;
use lykiadb_server::net::{CommunicationError, Message, Request};
use tokio::net::TcpStream;

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

    async fn execute(&mut self, query: &str) -> Result<Message, ()> {
        self.send_receive(Message::Request(Request::Run(query.to_string())))
            .await
    }
}
