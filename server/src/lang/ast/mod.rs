use std::rc::Rc;
use serde::Serialize;

use rustc_hash::FxHashMap;

use self::{
    expr::{Expr, ExprId},
    sql::{SqlCollectionSubquery, SqlExpr, SqlSelect, SqlSelectCore, SqlInsert, SqlUpdate, SqlDelete},
    stmt::{Stmt, StmtId},
};
pub mod expr;
pub mod sql;
pub mod stmt;
pub mod parser;
pub mod program;

#[derive(Debug, Clone, PartialEq, Serialize)]
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

impl Literal {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Literal::Str(s) => Some(s),
            _ => None,
        }
    }
}

impl Eq for Literal {}

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