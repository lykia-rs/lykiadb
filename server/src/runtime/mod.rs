use ::std::sync::Arc;

use self::environment::EnvironmentArena;
use self::error::{report_error, ExecutionError};
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
mod error;
mod eval;
pub mod interpreter;
mod resolver;
mod std;
pub mod types;

pub struct Runtime {
    mode: RuntimeMode,
    out: Option<Shared<Output>>
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

    pub fn print_ast(&mut self, source: &str) -> Result<(), ParseError> {
        let tokens = Scanner::scan(source).unwrap();
        let program = Parser::parse(&tokens)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&program.to_json()).unwrap()
        );
        Ok(())
    }

    pub fn interpret(&mut self, source: &str) -> Result<RV, ExecutionError> {
        let tokens = Scanner::scan(source);
        if tokens.is_err() {
            let error = error::ExecutionError::Scan(tokens.err().unwrap());
            report_error("filename", source, error.clone());
            return Err(error);
        }
        let program = Parser::parse(&tokens.unwrap());
        if program.is_err() {
            let error = error::ExecutionError::Parse(program.err().unwrap());
            report_error("filename", source, error.clone());
            return Err(error);
        }
        let program_unw = program.unwrap();
        let arena: Arc<AstArena> = Arc::clone(&program_unw.arena);
        //
        let mut resolver = Resolver::new(arena.clone());
        resolver.resolve_stmt(program_unw.root);
        //
        let mut env_arena = EnvironmentArena::new();
        let env = env_arena.top();

        let native_fns = stdlib(self.out.clone());

        for (name, value) in native_fns {
            env_arena.declare(env, name.to_string(), value);
        }

        let mut interpreter = Interpreter::new(env_arena, env, arena, Arc::new(resolver));
        let out = interpreter.visit_stmt(program_unw.root);

        if self.mode == RuntimeMode::Repl {
            println!("{:?}", out);
        }

        if let Ok(val) = out {
            Ok(val)
        } else {
            let err = out.err().unwrap();
            match err {
                HaltReason::Return(rv) => Ok(rv),
                HaltReason::Error(interpret_err) => {
                    let error = error::ExecutionError::Interpret(interpret_err);
                    report_error("<script>", source, error.clone());
                    Err(error)
                }
            }
        }
    }
}
