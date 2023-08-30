use std::rc::Rc;
use crate::lang::parsing::ast::Visitor;
use crate::lang::parsing::parser::Parser;
use crate::lang::parsing::scanner::Scanner;
use crate::lang::execution::environment::Environment;
use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::RV;
use crate::lang::execution::std::out::Print;
use crate::lang::execution::std::time::{Bench, Clock};

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

        env.borrow_mut().declare("clock".to_string(), RV::Callable(Rc::new(Clock)));
        env.borrow_mut().declare("bench".to_string(), RV::Callable(Rc::new(Bench)));
        env.borrow_mut().declare("print".to_string(), RV::Callable(Rc::new(Print)));

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