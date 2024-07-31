use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    ast::{expr::Expr, stmt::Stmt},
    Locals,
};
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
            Expr::Variable {
                name: _,
                span: _,
                id,
            } => *id,
            Expr::Assignment {
                span: _,
                id,
                expr: _,
                dst: _,
            } => *id,
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
