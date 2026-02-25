use crate::interpreter::error::ExecutionError;
use crate::interpreter::{Interpreter, Output};
use ::std::time::Instant;
use lykiadb_common::comm::tcp::TcpConnection;
use lykiadb_common::comm::{CommunicationError, Message, Request, Response};
use tokio::net::TcpStream;
use tracing::{error, info};

use std::{collections::HashMap, sync::Arc};

use crate::{
    util::{Shared, alloc_shared},
    value::RV,
};
use lykiadb_common::testing::TestHandler;
use lykiadb_lang::SourceProcessor;

pub struct Connection<'v> {
    conn: TcpConnection,
    runtime: Runtime<'v>,
}

impl<'v> Connection<'v> {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            conn: TcpConnection::new(stream),
            runtime: Runtime::new(RuntimeMode::File, Interpreter::new(None, true)),
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

pub struct Runtime<'v> {
    mode: RuntimeMode,
    source_processor: SourceProcessor,
    interpreter: Interpreter<'v>,
}

#[derive(Eq, PartialEq)]
pub enum RuntimeMode {
    Repl,
    File,
}

impl<'v> Runtime<'v> {
    pub fn new(mode: RuntimeMode, interpreter: Interpreter<'v>) -> Runtime<'v> {
        Runtime {
            mode,
            interpreter,
            source_processor: SourceProcessor::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<RV<'v>, ExecutionError> {
        let program = Arc::from(self.source_processor.process(source)?);
        let out = self.interpreter.interpret(program);

        if self.mode == RuntimeMode::Repl {
            info!("{:?}", out);
        }

        out
    }
}

pub struct RuntimeTester<'v> {
    out: Shared<Output<'v>>,
    runtime: Runtime<'v>,
}

impl<'v> Default for RuntimeTester<'v> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'v> RuntimeTester<'v> {
    pub fn new() -> RuntimeTester<'v> {
        let out = alloc_shared(Output::new());

        RuntimeTester {
            out: out.clone(),
            runtime: Runtime::new(RuntimeMode::File, Interpreter::new(Some(out), true)),
        }
    }
}

impl<'v> TestHandler for RuntimeTester<'v> {
    fn run_case(&mut self, case_parts: Vec<String>, flags: HashMap<&str, &str>) {
        assert!(
            case_parts.len() > 1,
            "Expected at least one input/output pair"
        );

        let mut errors: Vec<ExecutionError> = vec![];

        let result = self.runtime.interpret(&case_parts[0]);

        if let Err(err) = result {
            errors.push(err);
        }

        for part in &case_parts[1..] {
            if let Some(stripped) = part.strip_prefix("err") {
                assert_eq!(
                    errors
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join("\n"),
                    stripped.trim()
                );
            } else if let Some(stripped) = part.strip_prefix('>') {
                let result = self.runtime.interpret(stripped.trim());

                if let Err(err) = result {
                    errors.push(err);
                }
            } else if flags.get("run") == Some(&"plan") {
                // TODO(vck): Remove this
                self.out
                    .write()
                    .unwrap()
                    .expect(vec![RV::Str(Arc::new(part.to_string()))]);
            } else {
                self.out
                    .write()
                    .unwrap()
                    .expect_str(part.split('\n').map(|x| x.to_string()).collect());
            }
        }
    }
}
