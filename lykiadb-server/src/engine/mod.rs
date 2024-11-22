use self::error::ExecutionError;
use crate::util::Shared;
use crate::value::RV;
use interpreter::ExecutionContext;
use lykiadb_lang::parser::Parser;
use lykiadb_lang::tokenizer::scanner::Scanner;
use serde_json::Value;
use tracing::info;

pub mod error;
pub mod interpreter;
mod stdlib;

pub struct Runtime {
    mode: RuntimeMode,
    context: Shared<ExecutionContext>,
}

#[derive(Eq, PartialEq)]
pub enum RuntimeMode {
    Repl,
    File,
}

impl Runtime {
    pub fn new(mode: RuntimeMode, context: Shared<ExecutionContext>) -> Runtime {
        Runtime {
            mode,
            context,
        }
    }

    pub fn ast(&mut self, source: &str) -> Result<Value, ExecutionError> {
        let tokens = Scanner::scan(source)?;
        let program = Parser::parse(&tokens)?;
        let json = program.to_json();
        Ok(json)
    }

    pub fn interpret(&mut self, source: &str) -> Result<RV, ExecutionError> {
        let out = self.context.write().unwrap().interpret(source);

        if self.mode == RuntimeMode::Repl {
            info!("{:?}", out);
        }

        out
    }
}
