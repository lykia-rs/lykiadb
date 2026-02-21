use crate::{
    engine::{error::ExecutionError, interpreter::HaltReason},
    plan::scope::Scope,
};

use lykiadb_lang::ast::{
    Spanned,
    expr::Expr,
    sql::SqlSelect,
    visitor::{ExprReducer, ExprVisitorNode},
};

use super::PlannerError;

pub struct SqlExprReducer<'exec> {
    subqueries: Vec<SqlSelect>,
    allow_subqueries: bool,
    scope: &'exec Scope,
}

impl<'exec> SqlExprReducer<'exec> {
    pub fn new(allow_subqueries: bool, scope: &'exec Scope) -> Self {
        Self {
            subqueries: vec![],
            allow_subqueries,
            scope,
        }
    }
}

impl<'exec> ExprReducer<SqlSelect, HaltReason<'exec>> for SqlExprReducer<'exec> {
    fn visit(&mut self, expr: &Expr, visit: ExprVisitorNode) -> Result<bool, HaltReason<'exec>> {
        if matches!(visit, ExprVisitorNode::In) {
            match expr {
                Expr::Get { .. } => {
                    // check if the reference resolves
                    // println!("/Expr::Get({:?} of {:?})/", name.name, object);
                }
                Expr::FieldPath { .. } => {
                    // check if the head resolves
                    /* println!(
                        "/Is Expr::FieldPath(head={:?}) path_valid={:?}/",
                        head.name,
                        self.scope.is_path_valid(head, tail)
                    ); */
                }
                Expr::Call { .. } => {
                    // check if the callee resolves
                    // println!("/Expr::Call({callee:?})/");
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

    fn finalize(&mut self) -> Result<Vec<SqlSelect>, HaltReason<'exec>> {
        Ok(self.subqueries.clone())
    }
}
