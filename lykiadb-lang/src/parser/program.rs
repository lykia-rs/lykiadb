use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    Locals, Scopes,
    ast::{expr::Expr, stmt::Stmt},
    tokenizer::scanner::Scanner,
};

use super::{ParseError, ParseResult, Parser, resolver::Resolver};
#[derive(Serialize, Deserialize)]
pub struct Program {
    root: Box<Stmt>,
    locals: Option<Locals>,
}

impl Program {
    pub fn new(root: Box<Stmt>) -> Program {
        Program { root, locals: None }
    }

    pub fn set_locals(&mut self, locals: Locals) {
        self.locals = Some(locals);
    }

    pub fn get_distance(&self, expr: &Expr) -> Option<usize> {
        let expr_id: usize = match expr {
            Expr::Variable { id, .. } | Expr::Assignment { id, .. } => *id,
            _ => panic!("Expected variable or assignment expression."),
        };

        self.locals.as_ref().unwrap().get(&expr_id).copied()
    }

    pub fn get_root(&self) -> Box<Stmt> {
        self.root.clone()
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self.root.clone()).unwrap()
    }
}

impl FromStr for Program {
    type Err = ParseError;

    fn from_str(s: &str) -> ParseResult<Program> {
        let tokens = Scanner::scan(s).unwrap();
        let parse_result = Parser::parse(&tokens);

        if let Ok(mut program) = parse_result {
            let mut resolver = Resolver::new(Scopes::default(), &program, None);
            let (_, locals) = resolver.resolve().unwrap();
            program.set_locals(locals);
            return Ok(program);
        }
        panic!("Failed to parse program.");
    }
}
