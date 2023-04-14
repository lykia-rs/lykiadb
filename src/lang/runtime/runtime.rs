use crate::lang::parsing::ast::{Visitor};
use crate::lang::parsing::parser::Parser;
use crate::lang::parsing::scanner::Scanner;
use crate::lang::runtime::environment::Environment;
use crate::lang::runtime::interpreter::Interpreter;

pub struct Runtime {
    interpreter: Interpreter
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            interpreter: Interpreter::new(Environment::new())
        }
    }

    pub fn interpret(&mut self, source: &str) {
        let tokens = Scanner::scan(source);
        let ast = Parser::parse(&tokens);
        for stmt in ast {
            println!("{:?}", self.interpreter.visit_stmt(&stmt));
        }
    }
}