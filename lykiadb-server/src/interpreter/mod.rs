use crate::error::ExecutionError;
use crate::interpreter::environment::EnvironmentFrame;
use crate::interpreter::error::InterpretError;
use crate::interpreter::loops::{LoopStack, LoopState};
use crate::interpreter::output::Output;
use crate::value::RV;
use lykiadb_common::memory::Shared;
use std::sync::Arc;

pub mod environment;
pub mod error;
mod loops;
pub mod output;

use lykiadb_lang::ast::expr::{Expr, Operation, RangeKind};
use lykiadb_lang::ast::stmt::Stmt;
use lykiadb_lang::ast::visitor::VisitorMut;
use lykiadb_lang::ast::{Identifier, Literal, Spanned};
use lykiadb_lang::parser::program::Program;
use lykiadb_lang::types::Datatype;
use rustc_hash::FxHashMap;

use crate::global::GLOBAL_INTERNER;
use crate::libs::stdlib::stdlib;
use crate::query::exec::PlanExecutor;
use crate::query::plan::planner::Planner;
use crate::value::callable::{Function, RVCallable, Stateful};
use crate::value::eval::eval_binary;
use crate::value::iterator::ExecutionRow;
use crate::value::{array::RVArray, object::RVObject};
use interb::Symbol;
use std::vec;

#[derive(PartialEq, Debug)]
pub enum HaltReason<'v> {
    Error(ExecutionError),
    Return(RV<'v>),
}

#[derive(Clone)]
pub struct Interpreter<'sess> {
    env: Arc<EnvironmentFrame<'sess>>,
    root_env: Arc<EnvironmentFrame<'sess>>,
    exec_row: Option<ExecutionRow<'sess>>,
    //
    program: Option<Arc<Program>>,
    output: Option<Shared<Output<'sess>>>,
    //
    loop_stack: LoopStack,
}

impl<'sess> Interpreter<'sess> {
    pub fn eval_with_exec_row(
        &mut self,
        e: &Expr,
        exec_row: &ExecutionRow<'sess>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        self.set_exec_row(exec_row.clone());
        let evaluated = self.visit_expr(e);
        self.clear_exec_row();
        evaluated
    }

    fn get_from_exec_row(&mut self, name: &str) -> Option<RV<'sess>> {
        if let Some(exec_row) = &self.exec_row {
            if let Some(val) = exec_row.get(&self.intern_string(name)) {
                return Some(val.clone());
            }
        }
        None
    }

    pub fn set_exec_row(&mut self, exec_row: ExecutionRow<'sess>) {
        self.exec_row = Some(exec_row);
    }

    pub fn clear_exec_row(&mut self) {
        self.exec_row = None;
    }

    pub fn has_exec_row(&self) -> bool {
        self.exec_row.is_some()
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
            env: root_env.clone(),
            root_env,
            loop_stack: LoopStack::new(),
            program: None,
            output: out,
            exec_row: None,
        }
    }

    

    pub fn eval(&mut self, e: &Expr) -> Result<RV<'sess>, HaltReason<'sess>> {
        self.visit_expr(e)
    }

    pub fn interpret(&mut self, program: Arc<Program>) -> Result<RV<'sess>, ExecutionError> {
        self.program = Some(program.clone());
        let out = self.visit_stmt(&program.get_root());
        match out {
            Ok(val) => Ok(val),
            Err(err) => match err {
                HaltReason::Return(rv) => Ok(rv),
                HaltReason::Error(interpret_err) => Err(interpret_err),
            },
        }
    }

    fn eval_unary(
        &mut self,
        operation: &Operation,
        expr: &Expr,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        if *operation == Operation::Subtract {
            if let Some(num) = self.visit_expr(expr)?.as_double() {
                return Ok(RV::Double(-num));
            }
            Ok(RV::Undefined)
        } else {
            Ok(RV::Bool(!self.visit_expr(expr)?.as_bool()))
        }
    }

    fn eval_binary(
        &mut self,
        lexpr: &Expr,
        rexpr: &Expr,
        operation: Operation,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        let left_eval = self.visit_expr(lexpr)?;
        let right_eval = self.visit_expr(rexpr)?;

        Ok(eval_binary(left_eval, right_eval, operation))
    }

    fn intern_string(&self, string: &str) -> Symbol {
        GLOBAL_INTERNER.intern(string)
    }

    

    fn look_up_variable(
        &mut self,
        name: &str,
        expr: &Expr,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        if let Some(exec_row) = self.exec_row.as_ref()
            && let Some(val) = exec_row.get(&self.intern_string(name))
        {
            return Ok(val.clone());
        }

        let distance = self.program.as_ref().and_then(|x| x.get_distance(expr));
        if let Some(unwrapped) = distance {
            EnvironmentFrame::read_at(&self.env, unwrapped, name, &self.intern_string(name))
        } else {
            self.root_env.read(name, &self.intern_string(name))
        }
    }

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
        env_opt: Arc<EnvironmentFrame<'sess>>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        let previous = std::mem::replace(&mut self.env, env_opt);

        let mut ret = Ok(RV::Undefined);

        for statement in statements {
            ret = self.visit_stmt(statement);
            if ret.is_err() {
                break;
            }
        }

        self.env = previous;

        ret
    }

    fn literal_to_rv(&mut self, literal: &Literal) -> Result<RV<'sess>, HaltReason<'sess>> {
        Ok(match literal {
            Literal::Str(s) => RV::Str(Arc::clone(s)),
            Literal::Num(n) => RV::Double(*n),
            Literal::Bool(b) => RV::Bool(*b),
            Literal::Undefined => RV::Undefined,
            Literal::Object(map) => {
                let mut new_map = FxHashMap::default();
                for (k, v) in map.iter() {
                    new_map.insert(k.clone(), self.visit_expr(v)?);
                }
                RV::Object(RVObject::from_map(new_map))
            }
            Literal::Array(arr) => {
                let collected = arr
                    .iter()
                    .map(|x| self.visit_expr(x))
                    .collect::<Result<Vec<RV>, HaltReason>>()?;
                RV::Array(RVArray::from_vec(collected))
            }
        })
    }
}

impl<'sess> VisitorMut<RV<'sess>, HaltReason<'sess>> for Interpreter<'sess> {
    fn visit_expr(&mut self, e: &Expr) -> Result<RV<'sess>, HaltReason<'sess>> {
        match e {
            Expr::Literal { value, .. } => self.literal_to_rv(value),
            Expr::Variable { name, .. } => self.look_up_variable(&name.name, e),
            Expr::Unary {
                operation, expr, ..
            } => self.eval_unary(operation, expr),
            Expr::Binary {
                operation,
                left,
                right,
                ..
            } => self.eval_binary(left, right, *operation),
            Expr::Grouping { expr, .. } => self.visit_expr(expr),
            Expr::Logical {
                left,
                operation,
                right,
                ..
            } => {
                let is_true = self.visit_expr(left)?.as_bool();

                if (*operation == Operation::Or && is_true)
                    || (*operation == Operation::And && !is_true)
                {
                    return Ok(RV::Bool(is_true));
                }

                Ok(RV::Bool(self.visit_expr(right)?.as_bool()))
            }
            Expr::Assignment { dst, expr, .. } => {
                let distance = self
                    .program
                    .as_ref()
                    .ok_or(HaltReason::Error(ExecutionError::Interpret(
                        InterpretError::NoProgramLoaded,
                    )))?
                    .get_distance(e);

                let evaluated = self.visit_expr(expr)?;
                let dst_symbol = self.intern_string(&dst.name);
                if let Some(distance_unv) = distance {
                    EnvironmentFrame::assign_at(
                        &self.env,
                        distance_unv,
                        &dst.name,
                        dst_symbol,
                        evaluated.clone(),
                    )
                } else {
                    self.root_env
                        .assign(&dst.name, dst_symbol, evaluated.clone())
                }?;
                Ok(evaluated)
            }
            Expr::Call {
                callee, args, span, ..
            } => {
                let eval = self.visit_expr(callee)?;
                if let RV::Callable(callable) = eval {
                    if self.has_exec_row() && callable.is_agg() {
                        let value = self.get_from_exec_row(&e.sign());

                        if let Some(value) = value {
                            return Ok(value);
                        }

                        panic!("Aggregator value not found in execution row");
                    }

                    let mut args_evaluated: Vec<RV> = vec![];

                    for arg in args.iter() {
                        args_evaluated.push(self.visit_expr(arg)?);
                    }
                    self.loop_stack.push_fn();

                    let val = callable.call(self, span, args_evaluated.as_slice());

                    self.loop_stack.pop_fn();

                    match val {
                        Err(HaltReason::Return(ret_val)) => Ok(ret_val),
                        Ok(unpacked_val) => Ok(unpacked_val),
                        other_err @ Err(_) => other_err,
                    }
                } else {
                    Err(HaltReason::Error(
                        InterpretError::NotCallable {
                            span: callee.get_span(),
                        }
                        .into(),
                    ))
                }
            }
            Expr::Function {
                name,
                parameters,
                body,
                ..
            } => {
                let fn_name = if let Some(Identifier { name, .. }) = name {
                    name
                } else {
                    "<anonymous>"
                };

                let param_identifiers = parameters
                    .iter()
                    .map(|(x, _)| self.intern_string(&x.name))
                    .collect();

                let fun = Function::UserDefined {
                    name: self.intern_string(fn_name),
                    body: Arc::clone(body),
                    parameters: param_identifiers,
                    closure: self.env.clone(),
                };

                // TODO(vck): Type evaluation should be moved to a pre-execution phase
                let callable = RV::Callable(RVCallable::new(fun, Datatype::Unit, Datatype::Unit));

                if let Some(Identifier { name, .. }) = name {
                    // TODO(vck): Callable shouldn't be cloned here
                    self.env.define(self.intern_string(name), callable.clone());
                }

                Ok(callable)
            }
            Expr::Between {
                lower,
                upper,
                subject,
                kind,
                span,
                ..
            } => {
                let lower_eval = self.visit_expr(lower)?;
                let upper_eval = self.visit_expr(upper)?;
                let subject_eval = self.visit_expr(subject)?;

                if let (RV::Double(lower_num), RV::Double(upper_num), RV::Double(subject_num)) =
                    (lower_eval.clone(), upper_eval.clone(), subject_eval.clone())
                {
                    let min_num = lower_num.min(upper_num);
                    let max_num = lower_num.max(upper_num);

                    match kind {
                        RangeKind::Between => {
                            Ok(RV::Bool(min_num <= subject_num && subject_num <= max_num))
                        }
                        RangeKind::NotBetween => {
                            Ok(RV::Bool(min_num > subject_num || subject_num > max_num))
                        }
                    }
                } else {
                    Err(HaltReason::Error(
                        InterpretError::InvalidRangeExpression { span: *span }.into(),
                    ))
                }
            }
            Expr::FieldPath {
                head,
                tail,
                span,
                id,
            } => {
                let root = self.look_up_variable(&head.name, e);

                if tail.is_empty() {
                    return root;
                }

                let mut current = root?;

                for field in tail {
                    if let RV::Object(map) = current {
                        let v = map.get(&field.name);
                        if let Some(v) = v {
                            current = v;
                        } else {
                            return Err(HaltReason::Error(
                                InterpretError::PropertyNotFound {
                                    span: *span,
                                    property: field.name.to_string(),
                                }
                                .into(),
                            ));
                        }
                    } else {
                        return Err(HaltReason::Error(
                            InterpretError::InvalidPropertyAccess {
                                span: *span,
                                value_str: current.to_string(),
                            }
                            .into(),
                        ));
                    }
                }

                Ok(current)
            }
            Expr::Get {
                object, name, span, ..
            } => {
                let object_eval = self.visit_expr(object)?;
                if let RV::Object(map) = object_eval {
                    let v = map.get(&name.name.clone());
                    if let Some(v) = v {
                        return Ok(v.clone());
                    }
                    Err(HaltReason::Error(
                        InterpretError::PropertyNotFound {
                            span: *span,
                            property: name.name.to_string(),
                        }
                        .into(),
                    ))
                } else {
                    Err(HaltReason::Error(
                        InterpretError::InvalidPropertyAccess {
                            span: *span,
                            value_str: object_eval.to_string(),
                        }
                        .into(),
                    ))
                }
            }
            Expr::Set {
                object,
                name,
                value,
                span,
                ..
            } => {
                let object_eval = self.visit_expr(object)?;
                if let RV::Object(mut map) = object_eval {
                    let evaluated = self.visit_expr(value)?;
                    map.insert(name.name.to_string(), evaluated.clone());
                    Ok(evaluated)
                } else {
                    Err(HaltReason::Error(
                        InterpretError::InvalidPropertyAccess {
                            span: *span,
                            value_str: object_eval.to_string(),
                        }
                        .into(),
                    ))
                }
            }
            Expr::Select { .. }
            | Expr::Insert { .. }
            | Expr::Update { .. }
            | Expr::Delete { .. } => {
                let mut planner = Planner::new(self);
                let plan = planner.build(e)?;
                if let Some(out) = &self.output {
                    out.write()
                        .unwrap()
                        .push(RV::Str(Arc::new(plan.to_string().trim().to_string())));
                }
                let mut executor = PlanExecutor::new(self);
                let result = executor.execute_plan(plan);

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
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) -> Result<RV<'sess>, HaltReason<'sess>> {
        if !self.loop_stack.is_loops_empty()
            && *self.loop_stack.get_last_loop().unwrap() != LoopState::Go
        {
            return Ok(RV::Undefined);
        }

        match s {
            Stmt::Program { body: stmts, .. } => {
                return self.execute_block(stmts, self.env.clone());
            }
            Stmt::Expression { expr, .. } => {
                return self.visit_expr(expr);
            }
            Stmt::Declaration { dst, expr, .. } => {
                let evaluated = self.visit_expr(expr)?;
                self.env.define(self.intern_string(&dst.name), evaluated);
            }
            Stmt::Block { body: stmts, .. } => {
                return self.execute_block(
                    stmts,
                    Arc::new(EnvironmentFrame::new(Some(Arc::clone(&self.env)))),
                );
            }
            Stmt::If {
                condition,
                body,
                r#else_body: r#else,
                ..
            } => {
                if self.visit_expr(condition)?.as_bool() {
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
                self.loop_stack.push_loop(LoopState::Go);
                while !self.loop_stack.is_loop_at(LoopState::Broken)
                    && (condition.is_none()
                        || self.visit_expr(condition.as_ref().unwrap())?.as_bool())
                {
                    self.visit_stmt(body)?;
                    self.loop_stack
                        .set_loop_state(LoopState::Go, Some(LoopState::Continue));
                    if let Some(post_id) = post {
                        self.visit_stmt(post_id)?;
                    }
                }
                self.loop_stack.pop_loop();
            }
            Stmt::Break { span } => {
                if !self.loop_stack.set_loop_state(LoopState::Broken, None) {
                    return Err(HaltReason::Error(
                        InterpretError::UnexpectedStatement { span: *span }.into(),
                    ));
                }
            }
            Stmt::Continue { span } => {
                if !self.loop_stack.set_loop_state(LoopState::Continue, None) {
                    return Err(HaltReason::Error(
                        InterpretError::UnexpectedStatement { span: *span }.into(),
                    ));
                }
            }
            Stmt::Return { expr, .. } => {
                if let Some(expr) = expr {
                    let ret = self.visit_expr(expr)?;
                    return Err(HaltReason::Return(ret));
                }
                return Err(HaltReason::Return(RV::Undefined));
            }
            Stmt::Explain { expr, span } => {
                if matches!(expr.as_ref(), Expr::Select { .. }) {
                    let mut planner = Planner::new(self);
                    let plan = planner.build(expr)?;
                    if let Some(out) = &self.output {
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
