use crate::interpreter::HaltReason;
use crate::value::RV;
use crate::value::iterator::ExecutionRow;
use crate::{execution::state::ProgramState, interpreter::expr::ExprEngine};
use lykiadb_lang::ast::expr::Expr;

#[derive(Clone)]
pub struct QueryExecutionContext<'sess> {
    state: ProgramState<'sess>,
}

impl<'sess> QueryExecutionContext<'sess> {
    pub fn new(state: ProgramState<'sess>) -> Self {
        Self { state }
    }

    pub fn eval(&self, e: &Expr) -> Result<RV<'sess>, HaltReason<'sess>> {
        ExprEngine.eval(e, &self.state)
    }

    pub fn push_row(&self, row: &ExecutionRow<'sess>) {
        for (k, v) in row.keys.iter().zip(row.values.iter()) {
            self.state.env.define(*k, v.clone());
        }
    }

    pub fn pop_row(&self) {
        self.state.env.reset();
    }
}
