use crate::engine::interpreter::Interpreter;
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
            runtime: Runtime::new(RuntimeMode::File, Interpreter::new(None, true)),
        }
    }

    pub async fn handle(&mut self) {
        while let Some(message) = self.conn.read().await.unwrap() {
            // Here we measure the time it takes to process a message
            let start = Instant::now();
            match &message {
                Message::Request(req) => match req {
                    Request::Run(command) => {
                        let execution = self.runtime.interpret(command);

                        let response = if execution.is_ok() {
                            Response::Value(execution.unwrap().to_string().into())
                        } else {
                            Response::Error(execution.err().unwrap().generalize())
                        };

                        self.conn.write(Message::Response(response)).await.unwrap();
                    }
                },
                _ => error!("Unsupported message type"),
            }
            let elapsed = start.elapsed();
            info!("{:?} (took {:?})", message, elapsed);
        }
    }

    pub async fn send(&mut self, msg: Message) -> Result<(), CommunicationError> {
        self.conn.write(msg).await
    }
}
