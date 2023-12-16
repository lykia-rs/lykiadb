use self::error::{report_error, ExecutionError};
use self::interpreter::HaltReason;
use self::resolver::Resolver;
use self::std::stdlib;
use crate::lang::ast::Visitor;
use crate::lang::parser::Parser;
use crate::lang::scanner::Scanner;
use crate::runtime::environment::Environment;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::types::RV;
use crate::util::Shared;
use ::std::rc::Rc;

pub mod environment;
mod error;
mod eval;
pub mod interpreter;
mod resolver;
mod std;
mod tests;
pub mod types;

pub struct Runtime {
    env: Shared<Environment>,
    mode: RuntimeMode,
}

#[derive(Eq, PartialEq)]
pub enum RuntimeMode {
    Repl,
    File,
}

impl Runtime {
    pub fn new(mode: RuntimeMode) -> Runtime {
        let env = Environment::new(None);

        let native_fns = stdlib();

        for (name, value) in native_fns {
            env.borrow_mut().declare(name.to_string(), value);
        }

        Runtime { env, mode }
    }

    pub fn print_ast(&mut self, source: &str) {
        let tokens = Scanner::scan(source).unwrap();
        let parsed = Parser::parse(&tokens);
        println!("{}", parsed.unwrap().serialize());
    }

    pub fn interpret(&mut self, source: &str) -> Result<RV, ExecutionError> {
        let tokens = Scanner::scan(source);
        if tokens.is_err() {
            let error = error::ExecutionError::Scan(tokens.err().unwrap());
            report_error("filename", source, error.clone());
            return Err(error);
        }
        let parsed = Parser::parse(&tokens.unwrap());
        if parsed.is_err() {
            let error = error::ExecutionError::Parse(parsed.err().unwrap());
            report_error("filename", source, error.clone());
            return Err(error);
        }
        let parsed_unw = parsed.unwrap();
        let arena = Rc::clone(&parsed_unw.arena);
        //
        let mut resolver = Resolver::new(arena.clone());
        resolver.resolve_stmt(parsed_unw.program);
        //
        let mut interpreter = Interpreter::new(self.env.clone(), arena, Rc::new(resolver));
        let out = interpreter.visit_stmt(parsed_unw.program);

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
