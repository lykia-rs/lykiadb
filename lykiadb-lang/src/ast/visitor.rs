use super::{expr::Expr, stmt::Stmt};

pub trait Visitor<O, E> {
    fn visit_expr(&self, e: &Expr) -> Result<O, E>;
    fn visit_stmt(&self, s: &Stmt) -> Result<O, E>;
}
pub trait VisitorMut<O, E> {
    fn visit_expr(&mut self, e: &Expr) -> Result<O, E>;
    fn visit_stmt(&mut self, s: &Stmt) -> Result<O, E>;
}
