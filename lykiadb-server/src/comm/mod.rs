use crate::engine::{Runtime, RuntimeMode};
use ::std::time::Instant;
use lykiadb_common::comm::tcp::TcpConnection;
use lykiadb_common::comm::{CommunicationError, Message, Request, Response};
use tokio::net::TcpStream;
use tracing::{error, info};

pub struct ServerSession {
    conn: TcpConnection,
    runtime: Runtime,
}

impl ServerSession {
    pub fn new(stream: TcpStream) -> Self {
        ServerSession {
            conn: TcpConnection::new(stream),
            runtime: Runtime::new(RuntimeMode::File),
        }
    }

    pub async fn handle(&mut self) {
        while let Some(message) = self.conn.read().await.unwrap() {
            // Here we measure the time it takes to process a message

            match &message {
                Message::Request(req) => match req {
                    Request::Run(command) => {
                        let start = Instant::now();
                        let execution = self.runtime.interpret(command);
                        let elapsed = start.elapsed();
                        info!("{:?} (took {:?})", message, elapsed);
                        let response = if execution.is_ok() {
                            Response::Value(
                                execution.unwrap().to_string().into(),
                                elapsed.as_millis() as u64,
                            )
                        } else {
                            Response::Error(
                                execution.err().unwrap().generalize(),
                                elapsed.as_millis() as u64,
                            )
                        };

                        self.conn.write(Message::Response(response)).await.unwrap();
                    }
                },
                _ => error!("Unsupported message type"),
            }
        }
    }

    pub async fn send(&mut self, msg: Message) -> Result<(), CommunicationError> {
        self.conn.write(msg).await
    }
}
