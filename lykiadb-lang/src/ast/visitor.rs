use super::{
    expr::Expr,
    sql::{
        SqlCollectionSubquery, SqlDelete, SqlExpr, SqlInsert, SqlSelect, SqlSelectCore, SqlUpdate,
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

pub trait SqlVisitor<T, Q> {
    fn visit_sql_select(&self, e: &SqlSelect) -> Result<T, Q>;
    fn visit_sql_insert(&self, sql_insert: &SqlInsert) -> Result<T, Q>;
    fn visit_sql_update(&self, sql_update: &SqlUpdate) -> Result<T, Q>;
    fn visit_sql_delete(&self, sql_delete: &SqlDelete) -> Result<T, Q>;
    //
    fn visit_sql_select_core(&self, core: &SqlSelectCore) -> Result<T, Q>;
    fn visit_sql_subquery(&self, subquery: &SqlCollectionSubquery) -> Result<T, Q>;
    fn visit_sql_expr(&self, sql_expr: &SqlExpr) -> Result<T, Q>;
}

pub trait SqlVisitorMut<T, Q> {
    fn visit_sql_select(&mut self, e: &SqlSelect) -> Result<T, Q>;
    fn visit_sql_insert(&mut self, sql_insert: &SqlInsert) -> Result<T, Q>;
    fn visit_sql_update(&mut self, sql_update: &SqlUpdate) -> Result<T, Q>;
    fn visit_sql_delete(&mut self, sql_delete: &SqlDelete) -> Result<T, Q>;
    //
    fn visit_sql_select_core(&mut self, core: &SqlSelectCore) -> Result<T, Q>;
    fn visit_sql_subquery(&mut self, subquery: &SqlCollectionSubquery) -> Result<T, Q>;
    fn visit_sql_expr(&mut self, sql_expr: &SqlExpr) -> Result<T, Q>;
}
