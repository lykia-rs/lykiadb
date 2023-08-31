use std::rc::Rc;
use crate::lang::parsing::ast::Visitor;
use crate::lang::parsing::parser::Parser;
use crate::lang::parsing::scanner::Scanner;
use crate::lang::execution::environment::Environment;
use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{Function, RV};
use crate::lang::execution::std::fib::nt_fib;
use crate::lang::execution::std::out::nt_print;
use crate::lang::execution::std::time::nt_clock;
use super::std::json::{nt_json_encode, nt_json_decode};

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