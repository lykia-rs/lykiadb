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

    pub fn eval_with_exec_row(
        &self,
        e: &Expr,
        exec_row: ExecutionRow<'sess>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        self.state.exec_row.write().unwrap().replace(exec_row);
        let evaluated = self.eval(e);
        self.state.exec_row.write().unwrap().take();
        evaluated
    }
}
