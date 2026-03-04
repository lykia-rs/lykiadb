use crate::interpreter::HaltReason;
use crate::{interpreter::expr::ExprEngine, session::state::ProgramState};
use crate::value::RV;
use lykiadb_lang::ast::expr::{Expr};
use crate::value::iterator::ExecutionRow;

#[derive(Clone)]
pub struct ExecutionContext<'sess> {
    state: ProgramState<'sess>,
}

impl<'sess> ExecutionContext<'sess> {
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