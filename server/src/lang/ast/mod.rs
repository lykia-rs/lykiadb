use std::rc::Rc;

use rustc_hash::FxHashMap;

use self::{
    expr::{Expr, ExprId},
    stmt::{Stmt, StmtId},
};
pub mod expr;
pub mod sql;
pub mod stmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Str(Rc<String>),
    Num(f64),
    Bool(bool),
    Undefined,
    Object(FxHashMap<String, ExprId>),
    Array(Vec<ExprId>),
    NaN,
    Null,
}

impl Eq for Literal {}

pub trait Visitor<T, Q> {
    fn visit_expr(&mut self, e: ExprId) -> Result<T, Q>;
    fn visit_stmt(&mut self, e: StmtId) -> Result<T, Q>;
}

pub struct ParserArena {
    expressions: Vec<Expr>,
    statements: Vec<Stmt>,
}

impl ParserArena {
    pub fn new() -> ParserArena {
        ParserArena {
            expressions: Vec::new(),
            statements: Vec::new(),
        }
    }

    pub fn expression(&mut self, expr: Expr) -> ExprId {
        self.expressions.push(expr);
        ExprId(self.expressions.len() - 1)
    }

    pub fn statement(&mut self, stmt: Stmt) -> StmtId {
        self.statements.push(stmt);
        StmtId(self.statements.len() - 1)
    }

    pub fn get_expression(&self, idx: ExprId) -> &Expr {
        &self.expressions[idx.0]
    }

    pub fn get_statement(&self, idx: StmtId) -> &Stmt {
        &self.statements[idx.0]
    }
}
