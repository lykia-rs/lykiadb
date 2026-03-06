use crate::execution::error::ExecutionError;
use crate::execution::dispatching::dispatch_query_explain;
use crate::execution::state::ProgramState;
use crate::execution::global::intern_string;
use crate::interpreter::environment::EnvironmentFrame;
use crate::interpreter::expr::ExprEngine;
use crate::value::RV;
use std::sync::Arc;

pub mod environment;
pub mod error;
pub mod expr;
pub mod output;

use lykiadb_lang::ast::stmt::Stmt;
use lykiadb_lang::parser::program::Program;
use crate::value::callable::Stateful;
use interb::Symbol;

#[derive(PartialEq, Debug)]
pub enum HaltReason<'v> {
    Error(ExecutionError),
    Return(RV<'v>),
}

#[derive(Clone)]
pub struct Interpreter<'sess> {
    state: ProgramState<'sess>,
}

impl<'sess> Interpreter<'sess> {
    pub fn interpret(&mut self, program: Arc<Program>) -> Result<RV<'sess>, ExecutionError> {
        self.state.program = Some(program.clone());
        let out = self.visit_stmt(&program.get_root());
        match out {
            Ok(val) => Ok(val),
            Err(err) => match err {
                HaltReason::Return(rv) => Ok(rv),
                HaltReason::Error(interpret_err) => Err(interpret_err),
            },
        }
    }

    pub fn from_state(state: &ProgramState<'sess>) -> Interpreter<'sess> {
        Interpreter {
            state: state.clone(),
        }
    }
}

impl<'sess> Interpreter<'sess> {
    pub fn call_udf(
        &mut self,
        statements: &Vec<Stmt>,
        closure: Arc<EnvironmentFrame<'sess>>,
        parameters: &[Symbol],
        arguments: &[RV<'sess>],
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        let fn_env = EnvironmentFrame::new(Some(Arc::clone(&closure)));

        for (i, arg) in arguments.iter().enumerate() {
            if i >= parameters.len() {
                break;
            }

            // TODO: Remove clone here
            fn_env.define(parameters[i], arg.clone());
        }

        self.execute_block(statements, Arc::new(fn_env))
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        env_override: Arc<EnvironmentFrame<'sess>>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        let previous = std::mem::replace(&mut self.state.env, env_override);

        let mut ret = Ok(RV::Undefined);

        for statement in statements {
            ret = self.visit_stmt(statement);
            if ret.is_err() {
                break;
            }
        }

        self.state.env = previous;

        ret
    }

    fn visit_stmt(&mut self, s: &Stmt) -> Result<RV<'sess>, HaltReason<'sess>> {
        let expr_engine = ExprEngine;

        match s {
            Stmt::Program { body: stmts, .. } => {
                return self.execute_block(stmts, self.state.env.clone());
            }
            Stmt::Expression { expr, .. } => {
                return expr_engine.eval(expr, &self.state);
            }
            Stmt::Declaration { dst, expr, .. } => {
                let evaluated = expr_engine.eval(expr, &self.state)?;
                self.state.env.define(intern_string(&dst.name), evaluated);
            }
            Stmt::Block { body: stmts, .. } => {
                return self.execute_block(
                    stmts,
                    Arc::new(EnvironmentFrame::new(Some(Arc::clone(&self.state.env)))),
                );
            }
            Stmt::If {
                condition,
                body,
                r#else_body: r#else,
                ..
            } => {
                if expr_engine.eval(condition, &self.state)?.as_bool() {
                    self.visit_stmt(body)?;
                } else if let Some(else_stmt) = r#else {
                    self.visit_stmt(else_stmt)?;
                }
            }
            Stmt::Loop {
                condition,
                body,
                post,
                ..
            } => {
                while condition.is_none()
                    || expr_engine
                        .eval(condition.as_ref().unwrap(), &self.state)?
                        .as_bool()
                {
                    self.visit_stmt(body)?;
                    if let Some(post_id) = post {
                        self.visit_stmt(post_id)?;
                    }
                }
            }
            Stmt::Return { expr, .. } => {
                if let Some(expr) = expr {
                    let ret = expr_engine.eval(expr, &self.state)?;
                    return Err(HaltReason::Return(ret));
                }
                return Err(HaltReason::Return(RV::Undefined));
            }
            Stmt::Explain { expr, span } => return dispatch_query_explain(expr, span, self.state.clone()),
        }
        Ok(RV::Undefined)
    }
}

impl<'v> Stateful<'v> for output::Output<'v> {
    fn call(
        &mut self,
        _interpreter: &mut Interpreter<'v>,
        rv: &[RV<'v>],
    ) -> Result<RV<'v>, HaltReason<'v>> {
        for item in rv {
            self.push(item.clone());
        }
        Ok(RV::Undefined)
    }
}

#[cfg(test)]
pub mod tests {
    use lykiadb_common::memory::Shared;

    use crate::{execution::state::ProgramState, interpreter::{Interpreter, output::Output}};

    pub fn create_test_interpreter(out: Option<Shared<Output>>) -> Interpreter {
        let state = ProgramState::new(out, true);
        Interpreter::from_state(&state)
    }
}
