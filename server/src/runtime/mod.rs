use ::std::sync::Arc;

use serde_json::Value;
use tokio::net::TcpStream;
use tracing::{error, info};

use self::environment::Environment;
use self::error::ExecutionError;
use self::interpreter::{HaltReason, Output};
use self::resolver::Resolver;
use self::std::stdlib;
use crate::lang::ast::visitor::VisitorMut;

use crate::lang::ast::parser::Parser;
use crate::lang::ast::program::AstArena;
use crate::lang::tokens::scanner::Scanner;
use crate::net::{CommunicationError, Connection, Message, Request, Response};
use crate::runtime::interpreter::Interpreter;
use crate::runtime::types::RV;
use crate::util::Shared;

pub mod environment;
pub mod error;
pub mod interpreter;
mod resolver;
mod std;
pub mod types;

pub struct ServerSession {
    conn: Connection,
    runtime: Runtime,
}

impl ServerSession {
    pub fn new(stream: TcpStream) -> Self {
        ServerSession {
            conn: Connection::new(stream),
            runtime: Runtime::new(RuntimeMode::File),
        }
    }

    pub async fn handle(&mut self) {
        while let Some(message) = self.conn.read().await.unwrap() {
            info!("{:?}", message);
            match message {
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
        }
    }

    pub async fn send(&mut self, msg: Message) -> Result<(), CommunicationError> {
        self.conn.write(msg).await
    }
}

pub struct Runtime {
    mode: RuntimeMode,
    out: Option<Shared<Output>>,
}

#[derive(Eq, PartialEq)]
pub enum RuntimeMode {
    Repl,
    File,
}

impl Runtime {
    pub fn new(mode: RuntimeMode) -> Runtime {
        Runtime { mode, out: None }
    }

    pub fn ast(&mut self, source: &str) -> Result<Value, ExecutionError> {
        let tokens = Scanner::scan(source)?;
        let program = Parser::parse(&tokens)?;
        Ok(program.to_json())
    }

    pub fn interpret(&mut self, source: &str) -> Result<RV, ExecutionError> {
        let tokens = Scanner::scan(source)?;
        //
        let program = Parser::parse(&tokens)?;
        //
        let arena: Arc<AstArena> = Arc::clone(&program.arena);
        //
        let mut resolver = Resolver::new(arena.clone());
        resolver.resolve_stmt(program.root);
        //
        let mut env_man = Environment::new();
        let env = env_man.top();

        let native_fns = stdlib(self.out.clone());

        for (name, value) in native_fns {
            env_man.declare(env, name.to_string(), value);
        }

        let mut interpreter = Interpreter::new(env_man, env, arena, Arc::new(resolver));
        let out = interpreter.visit_stmt(program.root);

        if self.mode == RuntimeMode::Repl {
            info!("{:?}", out);
        }

        if let Ok(val) = out {
            Ok(val)
        } else {
            let err = out.err().unwrap();
            match err {
                HaltReason::Return(rv) => Ok(rv),
                HaltReason::Error(interpret_err) => {
                    let error = error::ExecutionError::Interpret(interpret_err);
                    Err(error)
                }
            }
        }
    }
}
