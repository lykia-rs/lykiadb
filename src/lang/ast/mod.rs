use self::{
    expr::{Expr, ExprId},
    stmt::{Stmt, StmtId},
};

pub mod expr;
pub mod sql;
pub mod stmt;

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
        self.expressions.len() - 1
    }

    pub fn statement(&mut self, stmt: Stmt) -> StmtId {
        self.statements.push(stmt);
        self.statements.len() - 1
    }

    pub fn get_expression(&self, idx: ExprId) -> &Expr {
        &self.expressions[idx]
    }

    pub fn get_statement(&self, idx: StmtId) -> &Stmt {
        &self.statements[idx]
    }
}
