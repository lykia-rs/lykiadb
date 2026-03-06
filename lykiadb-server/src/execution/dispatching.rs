use std::sync::Arc;

use crate::execution::state::ProgramState;
use crate::interpreter::HaltReason;
use crate::interpreter::error::InterpretError;
use crate::query::QueryEngine;
use crate::query::context::QueryExecutionContext;
use crate::value::RV;
use lykiadb_lang::ast::Span;
use lykiadb_lang::ast::expr::Expr;

pub fn dispatch_explain<'sess>(expr: &Box<Expr>, span: &Span, state: ProgramState<'sess>) -> Result<RV<'sess>, HaltReason<'sess>> {
    if matches!(expr.as_ref(), Expr::Select { .. }) {
        let output = &state.output;
        let exec_ctx = QueryExecutionContext::new(state.clone());
        let mut query_engine = QueryEngine::new();
        let plan = &query_engine.explain(expr, &exec_ctx)?;
        if let Some(out) = output {
            out.write()
                .unwrap()
                .push(RV::Str(Arc::new(plan.to_string().trim().to_string())));
        }
        return Err(HaltReason::Return(RV::Str(Arc::new(plan.to_string()))));
    } else {
        return Err(HaltReason::Error(
            InterpretError::InvalidExplainTarget { span: *span }.into(),
        ));
    }
}