use self::error::{report_error, ExecutionError};
use self::interpreter::HaltReason;
use self::resolver::Resolver;
use crate::lang::ast::Visitor;
use crate::lang::parser::Parser;
use crate::lang::scanner::Scanner;
use crate::runtime::environment::Environment;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::std::fib::nt_fib;
use crate::runtime::std::json::{nt_json_decode, nt_json_encode};
use crate::runtime::std::out::nt_print;
use crate::runtime::std::time::nt_clock;
use crate::runtime::types::{Function, RV};
use crate::util::Shared;
use ::std::collections::HashMap;
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

        let native_fns = HashMap::from([
            (
                "clock",
                RV::Callable(Some(0), Rc::new(Function::Lambda { function: nt_clock })),
            ),
            (
                "print",
                RV::Callable(None, Rc::new(Function::Lambda { function: nt_print })),
            ),
            (
                "fib_nat",
                RV::Callable(Some(1), Rc::new(Function::Lambda { function: nt_fib })),
            ),
            (
                "json_encode",
                RV::Callable(
                    Some(1),
                    Rc::new(Function::Lambda {
                        function: nt_json_encode,
                    }),
                ),
            ),
            (
                "json_decode",
                RV::Callable(
                    Some(1),
                    Rc::new(Function::Lambda {
                        function: nt_json_decode,
                    }),
                ),
            ),
        ]);

        for (name, value) in native_fns {
            env.borrow_mut().declare(name.to_string(), value);
        }

        Runtime { env, mode }
    }

    pub fn print_ast(&mut self, source: &str) {
        let tokens = Scanner::scan(source).unwrap();
        let parsed = Parser::parse(&tokens);
        println!("{:?}", parsed);
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
        let stmts = &parsed_unw.statements.clone();
        resolver.resolve_stmts(stmts);
        //
        let mut out = Ok(RV::Undefined);
        let mut interpreter = Interpreter::new(self.env.clone(), arena, Rc::new(resolver));
        for stmt in stmts {
            out = interpreter.visit_stmt(*stmt);
            if self.mode == RuntimeMode::Repl {
                println!("{:?}", out);
            }
            if out.is_err() {
                break;
            }
        }
        if out.is_err() {
            let err = out.err().unwrap();
            match err {
                HaltReason::Return(rv) => {
                    return Ok(rv);
                }
                HaltReason::Error(interpret_err) => {
                    let error = error::ExecutionError::Interpret(interpret_err);
                    report_error("filename", source, error.clone());
                    return Err(error);
                }
            }
        }
        else {
            return Ok(out.unwrap());
        }
    }
}
