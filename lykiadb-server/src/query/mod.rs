use lykiadb_lang::ast::expr::Expr;

use crate::{
    interpreter::HaltReason,
    query::{
        context::QueryExecutionContext, exec::PlanExecutor, plan::{Plan, planner::Planner}
    },
    value::{RV, array::RVArray, iterator::ExecutionRow},
};

pub mod exec;
pub mod plan;
pub mod context;

pub struct QueryEngine {
    planner: Planner,
    executor: PlanExecutor,
}

impl<'q> Default for QueryEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl<'q> QueryEngine {
    pub fn new() -> Self {
        QueryEngine {
            planner: Planner::new(),
            executor: PlanExecutor::new(),
        }
    }

    pub fn execute<'v>(
        &mut self,
        e: &Expr,
        exec_ctx: &'q QueryExecutionContext<'v>,
    ) -> Result<RV<'v>, HaltReason<'v>> {
        let plan = self.planner.build(e, exec_ctx)?;
        let result = self.executor.execute_plan(plan, exec_ctx);

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

    pub fn explain<'v>(
        &mut self,
        e: &Expr,
        exec_ctx: &'q QueryExecutionContext<'v>,
    ) -> Result<Plan<'v>, HaltReason<'v>> {
        self.planner.build(e, exec_ctx)
    }
}
