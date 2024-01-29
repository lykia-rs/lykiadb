use std::io::Error;

use liblykia::protocol::connection::{CommunicationError, Connection, Message, Request, Response};
use lykiadb_server::runtime::types::RV;
use lykiadb_server::runtime::{Runtime, RuntimeMode};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_stream::StreamExt as _;

struct Server {
    listener: Option<TcpListener>,
}

pub struct ServerSession {
    conn: Connection,
    runtime: Runtime
}

impl ServerSession {
    pub fn new(stream: TcpStream) -> Self {
        ServerSession {
            conn: Connection::new(stream),
            runtime: Runtime::new(RuntimeMode::File)
        }
    }

    pub async fn handle(&mut self) {
        while let Some(message) = self.conn.read().await.unwrap() {
            println!("{:?}", message);
            match message {
                Message::Request(req) => match req {
                    Request::Run(command) => {
                        let execution = self.runtime.interpret(&command);
                        if execution.is_ok() {
                            let bson_response = bson::to_bson(&execution.ok().or_else(|| Some(RV::Undefined)));
                            self.conn.write(Message::Response(Response::Value(bson_response.unwrap()))).await.unwrap();
                        }
                        else {
                            let err = execution.err().unwrap();
                            
                            self.conn.write(Message::Response(Response::Value(bson::to_bson(&format!("Error: {:?}", err)).unwrap()))).await.unwrap();
                        }
                    }
                },
                _ => panic!("Unsupported message type"),
            }
        }
    }

    pub async fn send(&mut self, msg: Message) -> Result<(), CommunicationError> {
        self.conn.write(msg).await
    }
}

impl Server {
    pub fn new() -> Result<Self, Error> {
        Ok(Server { listener: None })
    }

    pub async fn listen(mut self, addr: &str) -> Result<Self, Error> {
        let listener = TcpListener::bind(addr).await?;
        println!("Listening on {}", listener.local_addr()?);
        self.listener = Some(listener);
        Ok(self)
    }

    pub async fn serve(self) -> Result<(), Error> {
        if let Some(listener) = self.listener {
            let mut stream = TcpListenerStream::new(listener);
            while let Some(socket) = stream.try_next().await? {
                let peer = socket.peer_addr()?;
                tokio::spawn(async move {
                    let mut session = ServerSession::new(socket);
                    println!("Client {} connected", peer);
                    session.handle().await;
                    println!("Client {} disconnected", peer);
                });
            }
        }
        Ok(())
    }
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    Server::new()?.listen("0.0.0.0:19191").await?.serve().await
}
