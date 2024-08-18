use crate::value::types::RV;
use crate::engine::{Runtime, RuntimeMode};
use crate::net::tcp::TcpConnection;
use crate::net::{CommunicationError, Message, Request, Response};
use ::std::time::Instant;
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
            runtime: Runtime::new(RuntimeMode::File, None),
        }
    }

    pub async fn handle(&mut self) {
        while let Some(message) = self.conn.read().await.unwrap() {
            // Here we measure the time it takes to process a message
            let start = Instant::now();
            match &message {
                Message::Request(req) => match req {
                    Request::Ast(source) => {
                        let ast = self.runtime.ast(source);
                        self.conn
                            .write(Message::Response(Response::Program(ast.unwrap())))
                            .await
                            .unwrap();
                    }
                    Request::Run(command) => {
                        let execution = self.runtime.interpret(command);
                        let response = if execution.is_ok() {
                            Response::Value(execution.ok().or_else(|| Some(RV::Undefined)).unwrap())
                        } else {
                            Response::Error(execution.err().unwrap())
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
