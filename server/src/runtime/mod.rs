use ::std::sync::Arc;
use ::std::time::Instant;

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
use crate::util::{alloc_shared, Shared};

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
    out: Option<Shared<Output>>,
    arena: Option<AstArena>,
    env_man: Shared<Environment>,
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
            out,
            arena: Some(AstArena::new()),
            env_man: alloc_shared(env_man),
        }
    }

    pub fn ast(&mut self, source: &str) -> Result<Value, ExecutionError> {
        let tokens = Scanner::scan(source)?;
        let owned_arena = self.arena.take();
        let program = Parser::parse(&tokens, owned_arena.unwrap())?;
        let json = program.to_json();
        self.arena = Some(program.arena);
        Ok(json)
    }

    pub fn interpret(&mut self, source: &str) -> Result<RV, ExecutionError> {
        let tokens = Scanner::scan(source)?;
        let owned_arena = self.arena.take();
        let program = Parser::parse(&tokens, owned_arena.unwrap())?;
        self.arena = Some(program.arena);

        let mut resolver = Resolver::new(self.arena.as_ref().unwrap());
        resolver.resolve_stmt(program.root);

        /*
            TODO(vck): RwLock is probably an overkill here. Yet still, I couldn't find a better way to pass
            writable environment to the interpreter.
        */
        let env = self.env_man.as_ref().read().unwrap().top();
        let env_guard = self.env_man.as_ref().write().unwrap();

        let mut interpreter = Interpreter::new(
            env_guard,
            env,
            self.arena.as_ref().unwrap(),
            Arc::new(resolver),
        );

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
