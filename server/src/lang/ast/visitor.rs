use std::sync::Arc;

use super::{
    expr::ExprId,
    program::Program,
    sql::{
        SqlCollectionSubquery, SqlDelete, SqlExpr, SqlInsert, SqlSelect, SqlSelectCore, SqlUpdate,
    },
    stmt::StmtId,
};

pub trait Visitor<T, Q> {
    fn visit_expr(&self, program: Arc<Program>, e: ExprId) -> Result<T, Q>;
    fn visit_stmt(&self, program: Arc<Program>, e: StmtId) -> Result<T, Q>;
}

pub trait SqlVisitor<T, Q> {
    fn visit_sql_select(&self, program: Arc<Program>, e: &SqlSelect) -> Result<T, Q>;
    fn visit_sql_select_core(&self, program: Arc<Program>, core: &SqlSelectCore) -> Result<T, Q>;
    fn visit_sql_subquery(
        &self,
        program: Arc<Program>,
        subquery: &SqlCollectionSubquery,
    ) -> Result<T, Q>;
    fn visit_sql_expr(&self, program: Arc<Program>, sql_expr: &SqlExpr) -> Result<T, Q>;
    fn visit_sql_insert(&self, program: Arc<Program>, sql_insert: &SqlInsert) -> Result<T, Q>;
    fn visit_sql_update(&self, program: Arc<Program>, sql_update: &SqlUpdate) -> Result<T, Q>;
    fn visit_sql_delete(&self, program: Arc<Program>, sql_delete: &SqlDelete) -> Result<T, Q>;
}

pub trait VisitorMut<T, Q> {
    fn visit_expr(&mut self, program: Arc<Program>, e: ExprId) -> Result<T, Q>;
    fn visit_stmt(&mut self, program: Arc<Program>, e: StmtId) -> Result<T, Q>;
}

pub trait SqlVisitorMut<T, Q> {
    fn visit_sql_select(&mut self, program: Arc<Program>, e: &SqlSelect) -> Result<T, Q>;
    fn visit_sql_select_core(
        &mut self,
        program: Arc<Program>,
        core: &SqlSelectCore,
    ) -> Result<T, Q>;
    fn visit_sql_subquery(
        &mut self,
        program: Arc<Program>,
        subquery: &SqlCollectionSubquery,
    ) -> Result<T, Q>;
    fn visit_sql_expr(&mut self, program: Arc<Program>, sql_expr: &SqlExpr) -> Result<T, Q>;
    fn visit_sql_insert(&mut self, program: Arc<Program>, sql_insert: &SqlInsert) -> Result<T, Q>;
    fn visit_sql_update(&mut self, program: Arc<Program>, sql_update: &SqlUpdate) -> Result<T, Q>;
    fn visit_sql_delete(&mut self, program: Arc<Program>, sql_delete: &SqlDelete) -> Result<T, Q>;
}
