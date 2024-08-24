use super::{
    expr::Expr,
    sql::{
        SqlFrom, SqlDelete, SqlExpr, SqlInsert, SqlSelect, SqlSelectCore, SqlUpdate,
    },
    stmt::Stmt,
};

pub trait ExprEvaluator<O, E> {
    fn eval(&mut self, e: &Expr) -> Result<O, E>;
}

pub trait Visitor<O, E> {
    fn visit_expr(&self, e: &Expr) -> Result<O, E>;
    fn visit_stmt(&self, s: &Stmt) -> Result<O, E>;
}
pub trait VisitorMut<O, E> {
    fn visit_expr(&mut self, e: &Expr) -> Result<O, E>;
    fn visit_stmt(&mut self, s: &Stmt) -> Result<O, E>;
}
