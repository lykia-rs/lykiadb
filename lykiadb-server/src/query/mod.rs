use lykiadb_lang::ast::expr::Expr;

use crate::{interpreter::{HaltReason, expr::StatefulExprEngine}, query::{exec::PlanExecutor, plan::{Plan, planner::Planner}}};

pub mod exec;
pub mod plan;

pub struct QueryEngine<'v> {
    planner: Planner<'v>,
    executor: PlanExecutor,
}

impl<'v> QueryEngine<'v> {
    pub fn new(expr_engine: StatefulExprEngine<'v>) -> Self {
        QueryEngine {
            planner: Planner::new(expr_engine),
            executor: PlanExecutor::new(),
        }
    }

    pub fn explain(&mut self, expr: &Expr) -> Result<Plan<'v>, HaltReason<'v>> {
        self.planner.build(expr)
    }
}