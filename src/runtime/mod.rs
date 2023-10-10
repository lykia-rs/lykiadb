use ::std::collections::HashMap;
use ::std::rc::Rc;
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
use self::environment::Shared;

pub mod interpreter;
pub mod environment;
pub mod types;
mod eval;
mod std;
mod resolver;

pub struct Runtime {
    env: Shared<Environment>,
    mode: RuntimeMode
}

#[derive(Eq, PartialEq)]
pub enum RuntimeMode {
    Repl,
    File
}

impl Runtime {
    pub fn new(mode: RuntimeMode) -> Runtime {
        let env = Environment::new(None);

        let native_fns = HashMap::from([
            ("clock", RV::Callable(Some(0), Rc::new(Function::Native{ function: nt_clock }))),
            ("print", RV::Callable(None, Rc::new(Function::Native{ function: nt_print }))),
            ("fib_nat", RV::Callable(Some(1), Rc::new(Function::Native{ function: nt_fib }))),
            ("json_encode", RV::Callable(Some(1),Rc::new(Function::Native{ function: nt_json_encode }))),
            ("json_decode", RV::Callable(Some(1),Rc::new(Function::Native{ function: nt_json_decode }))),
        ]);

        for (name, value) in native_fns {
            env.borrow_mut().declare(name.to_string(), value);
        }

        Runtime {
            env,
            mode
        }
    }

    pub fn print_ast(&mut self, source: &str) {
        let tokens = Scanner::scan(source).unwrap();
        let parsed = Parser::parse(&tokens);
        println!("{:?}", parsed);
    }

    pub fn interpret(&mut self, source: &str) {
        let tokens = Scanner::scan(source).unwrap();
        let parsed = Parser::parse(&tokens);
        let arena = Rc::clone(&parsed.arena);
        let mut interpreter = Interpreter::new(self.env.clone(), arena);
        for stmt in parsed.statements.unwrap() {
            let out = interpreter.visit_stmt(stmt);
            if self.mode == RuntimeMode::Repl {
                println!("{:?}", out);
            }
        }
    }
}
