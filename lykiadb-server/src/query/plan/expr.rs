use crate::{
    interpreter::{error::ExecutionError, HaltReason},
    query::plan::scope::Scope,
};

use lykiadb_lang::ast::{
    Spanned,
    expr::Expr,
    sql::SqlSelect,
    visitor::{ExprReducer, ExprVisitorNode},
};

use super::PlannerError;

pub struct SqlExprReducer<'a> {
    subqueries: Vec<SqlSelect>,
    allow_subqueries: bool,
    scope: &'a Scope,
}

impl<'a> SqlExprReducer<'a> {
    pub fn new(allow_subqueries: bool, scope: &'a Scope) -> Self {
        Self {
            subqueries: vec![],
            allow_subqueries,
            scope,
        }
    }
}

impl<'scope, 'a> ExprReducer<SqlSelect, HaltReason<'a>> for SqlExprReducer<'scope> {
    fn visit(&mut self, expr: &Expr, visit: ExprVisitorNode) -> Result<bool, HaltReason<'a>> {
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

    fn finalize(&mut self) -> Result<Vec<SqlSelect>, HaltReason<'a>> {
        Ok(self.subqueries.clone())
    }
}
