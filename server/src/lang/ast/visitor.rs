use super::{expr::ExprId, stmt::StmtId, sql::{SqlSelect, SqlCollectionSubquery, SqlSelectCore, SqlExpr, SqlInsert, SqlUpdate, SqlDelete}};

pub trait Visitor<T, Q> {
    fn visit_expr(&self, e: ExprId) -> Result<T, Q>;
    fn visit_stmt(&self, e: StmtId) -> Result<T, Q>;
}

pub trait SqlVisitor<T, Q> {
    fn visit_sql_select(&self, e: &SqlSelect) -> Result<T, Q>;
    fn visit_sql_select_core(&self, core: &SqlSelectCore) -> Result<T, Q>;
    fn visit_sql_subquery(&self, subquery: &SqlCollectionSubquery) -> Result<T, Q>;
    fn visit_sql_expr(&self, sql_expr: &SqlExpr) -> Result<T, Q>;
    fn visit_sql_insert(&self, sql_insert: &SqlInsert) -> Result<T, Q>;
    fn visit_sql_update(&self, sql_update: &SqlUpdate) -> Result<T, Q>;
    fn visit_sql_delete(&self, sql_delete: &SqlDelete) -> Result<T, Q>;
}

pub trait VisitorMut<T, Q> {
    fn visit_expr(&mut self, e: ExprId) -> Result<T, Q>;
    fn visit_stmt(&mut self, e: StmtId) -> Result<T, Q>;
}

pub trait SqlVisitorMut<T, Q> {
    fn visit_sql_select(&mut self, e: &SqlSelect) -> Result<T, Q>;
    fn visit_sql_select_core(&mut self, core: &SqlSelectCore) -> Result<T, Q>;
    fn visit_sql_subquery(&mut self, subquery: &SqlCollectionSubquery) -> Result<T, Q>;
    fn visit_sql_expr(&mut self, sql_expr: &SqlExpr) -> Result<T, Q>;
    fn visit_sql_insert(&mut self, sql_insert: &SqlInsert) -> Result<T, Q>;
    fn visit_sql_update(&mut self, sql_update: &SqlUpdate) -> Result<T, Q>;
    fn visit_sql_delete(&mut self, sql_delete: &SqlDelete) -> Result<T, Q>;
}
