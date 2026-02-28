use lykiadb_lang::ast::expr::Expr;

use crate::{interpreter::{HaltReason, expr::StatefulExprEngine}, query::{exec::PlanExecutor, plan::{Plan, planner::Planner}}, value::{RV, array::RVArray, iterator::{ExecutionRow, RVs}}};

pub mod exec;
pub mod plan;

pub struct QueryEngine {
    planner: Planner,
    executor: PlanExecutor,
}

impl QueryEngine {
    pub fn new() -> Self {
        QueryEngine {
            planner: Planner::new(),
            executor: PlanExecutor::new(),
        }
    }

    pub fn execute<'v>(&mut self, e: &Expr, expr_engine: &'v StatefulExprEngine<'v>) -> Result<RV<'v>, HaltReason<'v>> {
        let plan = self.planner.build(e, expr_engine)?;
        let result = self.executor.execute_plan(plan, expr_engine);

        match result {
            Err(e) => Err(HaltReason::Error(e)),
            Ok(cursor) => {
                let intermediate = cursor
                    .map(|row: ExecutionRow| row.as_value())
                    .collect::<Vec<RV>>();
                Ok(RV::Array(RVArray::from_vec(intermediate)))
            }
        }
    }

    pub fn explain<'v>(&mut self, e: &Expr, expr_engine: &'v StatefulExprEngine<'v>) -> Result<Plan<'v>, HaltReason<'v>> {
        self.planner.build(e, expr_engine)
    }
}