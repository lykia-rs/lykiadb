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

pub mod interpreter;
pub mod environment;
pub mod types;
mod std;
mod resolver;

pub struct Runtime {
    interpreter: Interpreter,
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
        let mut interpreter = Interpreter::new(env);

        let native_fns = HashMap::from([
            ("clock", RV::Callable(Some(0), Rc::new(Function::Native{ function: nt_clock }))),
            ("print", RV::Callable(None, Rc::new(Function::Native{ function: nt_print }))),
            ("fib_nat", RV::Callable(Some(1), Rc::new(Function::Native{ function: nt_fib }))),
            ("json_encode", RV::Callable(Some(1),Rc::new(Function::Native{ function: nt_json_encode }))),
            ("json_decode", RV::Callable(Some(1),Rc::new(Function::Native{ function: nt_json_decode }))),
        ]);

        interpreter.define_native_fns(native_fns);

        Runtime {
            interpreter,
            mode
        }
    }

    pub fn interpret(&mut self, source: &str) {
        let tokens = Scanner::scan(source).unwrap();
        let stmts = Parser::parse(&tokens).unwrap();
        for stmt in stmts {
            let out = self.interpreter.visit_stmt(&stmt);
            if self.mode == RuntimeMode::Repl {
                println!("{:?}", out);
            }
        }
    }
}