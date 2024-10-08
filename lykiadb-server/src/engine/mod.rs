use self::error::ExecutionError;
use self::interpreter::Output;
use self::stdlib::stdlib;
use crate::engine::interpreter::Interpreter;
use crate::util::{alloc_shared, Shared};
use crate::value::environment::Environment;
use crate::value::types::RV;
use lykiadb_lang::parser::Parser;
use lykiadb_lang::tokenizer::scanner::Scanner;
use serde_json::Value;
use tracing::info;

pub mod error;
pub mod interpreter;
mod stdlib;

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
