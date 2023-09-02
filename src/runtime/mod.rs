use ::std::rc::Rc;
use crate::lang::ast::Visitor;
use crate::lang::parser::Parser;
use crate::lang::scanner::Scanner;
use crate::lang::types::RV;
use crate::runtime::environment::Environment;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::primitives::Function;
use crate::runtime::std::fib::nt_fib;
use crate::runtime::std::json::{nt_json_decode, nt_json_encode};
use crate::runtime::std::out::nt_print;
use crate::runtime::std::time::nt_clock;

pub mod interpreter;
mod environment;
mod primitives;
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

        env.borrow_mut().declare("clock".to_string(), RV::Callable(Some(0), Rc::new(Function::Native(nt_clock))));
        env.borrow_mut().declare("print".to_string(), RV::Callable(None, Rc::new(Function::Native(nt_print))));
        env.borrow_mut().declare("fib_nat".to_string(), RV::Callable(Some(1),Rc::new(Function::Native(nt_fib))));
        env.borrow_mut().declare("json_encode".to_string(), RV::Callable(Some(1),Rc::new(Function::Native(nt_json_encode))));
        env.borrow_mut().declare("json_decode".to_string(), RV::Callable(Some(1),Rc::new(Function::Native(nt_json_decode))));

        Runtime {
            interpreter: Interpreter::new(env),
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