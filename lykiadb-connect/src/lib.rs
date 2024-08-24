use crate::session::ClientSession;
use crate::tcp::TcpClientSession;
use tokio::net::TcpStream;

pub mod session;
mod tcp;

pub use lykiadb_server::engine::error::report_error;
pub use lykiadb_server::comm::{Message, Request, Response};

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
