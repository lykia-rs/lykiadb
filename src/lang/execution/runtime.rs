use std::rc::Rc;
use crate::lang::parsing::ast::{Visitor};
use crate::lang::parsing::parser::Parser;
use crate::lang::parsing::scanner::Scanner;
use crate::lang::execution::environment::{EnvironmentStack};
use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::RV;
use crate::lang::execution::std::time::Clock;

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
        let mut env = EnvironmentStack::new();

        env.declare("clock".to_string(), RV::Callable(Rc::new(Clock::new())));

        Runtime {
            interpreter: Interpreter::new(env),
            mode
        }
    }

    pub fn interpret(&mut self, source: &str) {
        let tokens = Scanner::scan(source);
        let ast = Parser::parse(&tokens);
        for stmt in ast {
            let out = self.interpreter.visit_stmt(&stmt);
            if self.mode == RuntimeMode::Repl {
                println!("{:?}", out);
            }
        }
    }
}