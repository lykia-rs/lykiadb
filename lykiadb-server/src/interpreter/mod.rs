use crate::error::ExecutionError;
use crate::interpreter::environment::EnvironmentFrame;
use crate::interpreter::error::InterpretError;
use crate::interpreter::expr::{ExprEngine, StatefulExprEngine};
use crate::interpreter::output::Output;
use crate::value::RV;
use crate::value::iterator::ExecutionRow;
use lykiadb_common::memory::Shared;
use std::sync::Arc;

pub mod environment;
pub mod error;
pub mod expr;
pub mod output;

use lykiadb_lang::ast::stmt::Stmt;
use lykiadb_lang::parser::program::Program;

use crate::global::GLOBAL_INTERNER;
use crate::libs::stdlib::stdlib;
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

#[derive(Clone)]
pub struct ProgramState<'sess> {
    pub env: Arc<EnvironmentFrame<'sess>>,
    pub exec_row: Shared<Option<ExecutionRow<'sess>>>,
    // Output
    pub output: Option<Shared<Output<'sess>>>,
    // Static fields:
    pub root_env: Arc<EnvironmentFrame<'sess>>,
    pub program: Option<Arc<Program>>,
}

impl<'sess> ProgramState<'sess> {
    pub fn new(env: Arc<EnvironmentFrame<'sess>>, root_env: Arc<EnvironmentFrame<'sess>>, program: Option<Arc<Program>>, output: Option<Shared<Output<'sess>>>) -> Self {
        Self {
            env,
            root_env,
            exec_row: Shared::new(None.into()),
            program,
            output,
        }
    }
}

impl<'sess> Interpreter<'sess> {
    pub fn new(out: Option<Shared<Output<'sess>>>, with_stdlib: bool) -> Interpreter<'sess> {
        let root_env = Arc::new(EnvironmentFrame::new(None));
        if with_stdlib {
            let native_fns = stdlib(out.clone());

            for (name, value) in native_fns {
                root_env.define(GLOBAL_INTERNER.intern(&name), value);
            }
        }
        Interpreter {
            state: ProgramState::new(
                root_env.clone(), 
                root_env.clone(), 
                None,
                 out),
        }
    }

    pub fn interpret(&mut self, program: Arc<Program>) -> Result<RV<'sess>, ExecutionError> {
        self.state.program = Some(program.clone());
        let out = self.visit_stmt(&program.get_root(), &self.state.clone());
        match out {
            Ok(val) => Ok(val),
            Err(err) => match err {
                HaltReason::Return(rv) => Ok(rv),
                HaltReason::Error(interpret_err) => Err(interpret_err),
            },
        }
    }

    pub fn get_expr_engine(&self) -> StatefulExprEngine<'sess> {
        StatefulExprEngine::new(self.state.clone())
    }
    
    pub fn from_state(state: &ProgramState<'sess>) -> Interpreter<'sess> {
        Interpreter {
            state: state.clone(),
        }
    }
}

impl<'sess> Interpreter<'sess> {
    pub fn user_fn_call(
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

        let state = self.state.clone();

        for statement in statements {
            ret = self.visit_stmt(statement, &state);
            if ret.is_err() {
                break;
            }
        }

        self.state.env = previous;

        ret
    }

    fn intern_string(&self, string: &str) -> Symbol {
        GLOBAL_INTERNER.intern(string)
    }

    fn visit_stmt(&mut self, s: &Stmt, state: &ProgramState<'sess>) -> Result<RV<'sess>, HaltReason<'sess>> {
        let expr_engine = ExprEngine;

        match s {
            Stmt::Program { body: stmts, .. } => {
                return self.execute_block(stmts, self.state.env.clone());
            }
            Stmt::Expression { expr, .. } => {
                return expr_engine.eval(expr, state);
            }
            Stmt::Declaration { dst, expr, .. } => {
                let evaluated = expr_engine.eval(expr, state)?;
                self.state.env.define(self.intern_string(&dst.name), evaluated);
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
                if expr_engine.eval(condition, state)?.as_bool() {
                    self.visit_stmt(body, state)?;
                } else if let Some(else_stmt) = r#else {
                    self.visit_stmt(else_stmt, state)?;
                }
            }
            Stmt::Loop {
                condition,
                body,
                post,
                ..
            } => {
                
                while condition.is_none()
                        || expr_engine.eval(condition.as_ref().unwrap(), state)?.as_bool()
                {
                    self.visit_stmt(body, state)?;
                    if let Some(post_id) = post {
                        self.visit_stmt(post_id, state)?;
                    }
                }
            }
            Stmt::Return { expr, .. } => {
                if let Some(expr) = expr {
                    let ret = expr_engine.eval(expr,state)?;
                    return Err(HaltReason::Return(ret));
                }
                return Err(HaltReason::Return(RV::Undefined));
            }
            Stmt::Explain { expr, span } => {
                // TODO(LYK-28)
                /* if matches!(expr.as_ref(), Expr::Select { .. }) {
                    let mut planner = Planner::new(self);
                    let plan = planner.build(expr)?;
                    if let Some(out) = &self.output {
                        out.write()
                            .unwrap()
                            .push(RV::Str(Arc::new(plan.to_string().trim().to_string())));
                    }
                    return Err(HaltReason::Return(RV::Str(Arc::new(plan.to_string()))));
                } else {*/
                    return Err(HaltReason::Error(
                        InterpretError::InvalidExplainTarget { span: *span }.into(),
                    ));
                //}
            }
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

    use crate::interpreter::{Interpreter, Output};

    pub fn create_test_interpreter(out: Option<Shared<Output>>) -> Interpreter {
        Interpreter::new(out, true)
    }
}
