use ::std::sync::Arc;

use serde_json::Value;
use tracing::info;

use self::environment::Environment;
use self::error::ExecutionError;
use self::interpreter::{HaltReason, Output};
use self::resolver::Resolver;
use self::std::stdlib;
use crate::lang::ast::visitor::VisitorMut;

use crate::lang::ast::parser::{ParseError, Parser};
use crate::lang::ast::program::AstArena;
use crate::lang::tokens::scanner::Scanner;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::types::RV;
use crate::util::Shared;

pub mod environment;
pub mod error;
pub mod interpreter;
mod resolver;
mod std;
pub mod types;

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
