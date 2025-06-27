use crate::engine::{error::ExecutionError, interpreter::HaltReason};

use lykiadb_lang::ast::{
    Spanned,
    expr::Expr,
    sql::SqlSelect,
    visitor::{ExprReducer, ExprVisitorNode},
};

use super::PlannerError;

pub struct SqlExprReducer {
    subqueries: Vec<SqlSelect>,
    allow_subqueries: bool,
}

impl<'a> SqlExprReducer {
    pub fn new(allow_subqueries: bool) -> Self {
        Self {
            subqueries: vec![],
            allow_subqueries,
        }
    }
}

impl<'a> ExprReducer<SqlSelect, HaltReason> for SqlExprReducer {
    fn visit(&mut self, expr: &Expr, visit: ExprVisitorNode) -> Result<bool, HaltReason> {
        if matches!(visit, ExprVisitorNode::In) {
            match expr {
                Expr::Get { object, name, .. } => {
                    // check if the reference resolves
                    println!("/Expr::Get({:?} of {:?})/", name.name, object);
                }
                Expr::FieldPath { head, .. } => {
                    // check if the head resolves
                    println!("/Expr::FieldPath(head={:?})/", head.name);
                }
                Expr::Call { callee, .. } => {
                    // check if the callee resolves
                    println!("/Expr::Call({callee:?})/");
                }
                Expr::Select { query, .. } => {
                    if !self.allow_subqueries {
                        return Err(HaltReason::Error(ExecutionError::Plan(
                            PlannerError::SubqueryNotAllowed(expr.get_span()),
                        )));
                    }
                    self.subqueries.push(query.clone());
                    return Ok(false);
                }
                _ => {}
            }
        }

        Ok(true)
    }

    fn finalize(&mut self) -> Result<Vec<SqlSelect>, HaltReason> {
        Ok(self.subqueries.clone())
    }
}
