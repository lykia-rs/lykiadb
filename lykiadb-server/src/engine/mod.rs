use self::environment::Environment;
use self::error::ExecutionError;
use self::interpreter::Output;
use self::std::stdlib;
use crate::engine::interpreter::Interpreter;
use crate::engine::types::RV;
use crate::lang::parser::Parser;
use crate::lang::tokenizer::scanner::Scanner;
use crate::util::{alloc_shared, Shared};
use serde_json::Value;
use tracing::info;

pub mod environment;
pub mod error;
pub mod interpreter;
mod std;
pub mod types;

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
