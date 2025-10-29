use tokio::net::TcpStream;

use crate::comm::{CommunicationError, Message, Request};
use crate::comm::tcp::TcpConnection;

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

pub trait ClientSession {
    async fn send_receive(&mut self, msg: Message) -> Result<Message, ()>;
    async fn execute(&mut self, query: &str) -> Result<Message, ()>;
}

pub enum Protocol {
    Tcp,
    Http,
}

pub async fn get_session(addr: &str, protocol: Protocol) -> impl ClientSession {
    match protocol {
        Protocol::Tcp => {
            let socket = TcpStream::connect(addr).await.unwrap();
            TcpClientSession::new(socket)
        }
        Protocol::Http => {
            panic!("Http not implemented!")
        }
    }
}
pub async fn connect(addr: &str) -> impl ClientSession {
    get_session(addr, Protocol::Tcp).await
}
