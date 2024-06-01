use ::std::time::Instant;

use serde_json::Value;
use tokio::net::TcpStream;
use tracing::{error, info};

use self::environment::Environment;
use self::error::ExecutionError;
use self::interpreter::Output;
use self::std::stdlib;

use crate::lang::parser::Parser;
use crate::lang::tokenizer::scanner::Scanner;
use crate::net::{CommunicationError, Message, Request, Response};
use crate::net::tcp::TcpConnection;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::types::RV;
use crate::util::{alloc_shared, Shared};

pub mod environment;
pub mod error;
pub mod interpreter;
mod std;
pub mod types;

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
                        let ast = self.runtime.ast(&source);
                        self.conn
                            .write(Message::Response(Response::Program(ast.unwrap())))
                            .await
                            .unwrap();
                    }
                    Request::Run(command) => {
                        let execution = self.runtime.interpret(&command);
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

pub struct Runtime {
    mode: RuntimeMode,
    interpreter: Interpreter,
}

#[derive(Eq, PartialEq)]
pub enum RuntimeMode {
    Repl,
    File,
}

impl Runtime {
    pub fn new(mode: RuntimeMode, out: Option<Shared<Output>>) -> Runtime {
        let mut env_man = Environment::new();
        let native_fns = stdlib(out.clone());
        let env = env_man.top();

        for (name, value) in native_fns {
            env_man.declare(env, name.to_string(), value);
        }
        Runtime {
            mode,
            interpreter: Interpreter::new(alloc_shared(env_man)),
        }
    }

    pub fn ast(&mut self, source: &str) -> Result<Value, ExecutionError> {
        let tokens = Scanner::scan(source)?;
        let program = Parser::parse(&tokens)?;
        let json = program.to_json();
        Ok(json)
    }

    pub fn interpret(&mut self, source: &str) -> Result<RV, ExecutionError> {
        let out = self.interpreter.interpret(source);

        if self.mode == RuntimeMode::Repl {
            info!("{:?}", out);
        }

        out
    }
}
