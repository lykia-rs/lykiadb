use crate::error::ExecutionError;
use crate::global::GLOBAL_INTERNER;
use crate::interpreter::HaltReason;
use crate::interpreter::environment::EnvironmentFrame;
use crate::interpreter::error::InterpretError;
use crate::interpreter::output::Output;
use crate::value::RV;
use crate::value::array::RVArray;
use crate::value::callable::{Function, RVCallable};
use crate::value::eval::eval_binary;
use crate::value::object::RVObject;
use std::sync::Arc;

use interb::Symbol;
use lykiadb_common::memory::Shared;
use lykiadb_lang::ast::{Identifier, Literal, Spanned};
use lykiadb_lang::ast::expr::{Expr, Operation, RangeKind};
use lykiadb_lang::parser::program::Program;
use lykiadb_lang::types::Datatype;
use rustc_hash::FxHashMap;
use crate::value::iterator::ExecutionRow;

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

#[derive(Clone)]
pub struct StatefulExprEngine<'sess> {
    state: ProgramState<'sess>,
}

impl<'sess> StatefulExprEngine<'sess> {
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

#[derive(Clone)]
pub struct ExprEngine;

impl<'sess> ExprEngine {
    fn get_from_exec_row(&self, name: &str, state: &ProgramState<'sess>) -> Option<RV<'sess>> {
        if let Some(exec_row) = &*state.exec_row.read().unwrap() {
            if let Some(val) = exec_row.get(&self.intern_string(name)) {
                return Some(val.clone());
            }
        }
        None
    }

    pub fn has_exec_row(&self, state: &ProgramState<'sess>) -> bool {
        state.exec_row.read().unwrap().is_some()
    }
}

impl<'sess> ExprEngine {
    pub fn eval(&self, e: &Expr, state: &ProgramState<'sess>) -> Result<RV<'sess>, HaltReason<'sess>> {
        self.visit_expr(e, state)
    }
}

impl<'sess> ExprEngine {
    fn eval_unary(
        &self,
        operation: &Operation,
        expr: &Expr,
        state: &ProgramState<'sess>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        if *operation == Operation::Subtract {
            if let Some(num) = self.eval(expr, state)?.as_double() {
                return Ok(RV::Double(-num));
            }
            Ok(RV::Undefined)
        } else {
            Ok(RV::Bool(!self.eval(expr, state)?.as_bool()))
        }
    }

    fn eval_binary(
        &self,
        lexpr: &Expr,
        rexpr: &Expr,
        operation: Operation,
        state: &ProgramState<'sess>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        let left_eval = self.eval(lexpr, state)?;
        let right_eval = self.eval(rexpr, state)?;

        Ok(eval_binary(left_eval, right_eval, operation))
    }

    fn intern_string(&self, string: &str) -> Symbol {
        GLOBAL_INTERNER.intern(string)
    }

    fn look_up_variable(
        &self,
        name: &str,
        expr: &Expr,
        state: &ProgramState<'sess>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        if let Some(exec_row) = &*state.exec_row.read().unwrap()
            && let Some(val) = exec_row.get(&self.intern_string(name))
        {
            return Ok(val.clone());
        }

        let distance = state.program.as_ref().and_then(|x| x.get_distance(expr));
        if let Some(unwrapped) = distance {
            EnvironmentFrame::read_at(&state.env, unwrapped, name, &self.intern_string(name))
        } else {
            state.root_env.read(name, &self.intern_string(name))
        }
    }

    fn literal_to_rv(&self, literal: &Literal, state: &ProgramState<'sess>) -> Result<RV<'sess>, HaltReason<'sess>> {
        Ok(match literal {
            Literal::Str(s) => RV::Str(Arc::clone(s)),
            Literal::Num(n) => RV::Double(*n),
            Literal::Bool(b) => RV::Bool(*b),
            Literal::Undefined => RV::Undefined,
            Literal::Object(map) => {
                let mut new_map = FxHashMap::default();
                for (k, v) in map.iter() {
                    new_map.insert(k.clone(), self.eval(v, state)?);
                }
                RV::Object(RVObject::from_map(new_map))
            }
            Literal::Array(arr) => {
                let collected = arr
                    .iter()
                    .map(|x| self.eval(x, state))
                    .collect::<Result<Vec<RV>, HaltReason>>()?;
                RV::Array(RVArray::from_vec(collected))
            }
        })
    }

    fn visit_expr(&self, e: &Expr, state: &ProgramState<'sess>) -> Result<RV<'sess>, HaltReason<'sess>> {
        match e {
            Expr::Literal { value, .. } => self.literal_to_rv(value, state),
            Expr::Variable { name, .. } => self.look_up_variable(&name.name, e, state),
            Expr::Unary {
                operation, expr, ..
            } => self.eval_unary(operation, expr, state),
            Expr::Binary {
                operation,
                left,
                right,
                ..
            } => self.eval_binary(left, right, *operation, state),
            Expr::Grouping { expr, .. } => self.eval(expr, state),
            Expr::Logical {
                left,
                operation,
                right,
                ..
            } => {
                let is_true = self.eval(left, state)?.as_bool();

                if (*operation == Operation::Or && is_true)
                    || (*operation == Operation::And && !is_true)
                {
                    return Ok(RV::Bool(is_true));
                }

                Ok(RV::Bool(self.eval(right, state)?.as_bool()))
            }
            Expr::Assignment { dst, expr, .. } => {
                let distance = state
                    .program
                    .as_ref()
                    .ok_or(HaltReason::Error(ExecutionError::Interpret(
                        InterpretError::NoProgramLoaded,
                    )))?
                    .get_distance(e);

                let evaluated = self.eval(expr, state)?;
                let dst_symbol = self.intern_string(&dst.name);
                if let Some(distance_unv) = distance {
                    EnvironmentFrame::assign_at(
                        &state.env,
                        distance_unv,
                        &dst.name,
                        dst_symbol,
                        evaluated.clone(),
                    )
                } else {
                    state.root_env
                        .assign(&dst.name, dst_symbol, evaluated.clone())
                }?;
                Ok(evaluated)
            }
            Expr::Call {
                callee, args, span, ..
            } => {
                let eval = self.eval(callee, state)?;
                if let RV::Callable(callable) = eval {
                    if self.has_exec_row(state) && callable.is_agg() {
                        let value = self.get_from_exec_row(&e.sign(), state);

                        if let Some(value) = value {
                            return Ok(value);
                        }

                        panic!("Aggregator value not found in execution row");
                    }

                    let mut args_evaluated: Vec<RV> = vec![];

                    for arg in args.iter() {
                        args_evaluated.push(self.eval(arg, state)?);
                    }

                    let val = callable.call(state, span, args_evaluated.as_slice());

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
                    closure: state.env.clone(),
                };

                // TODO(vck): Type evaluation should be moved to a pre-execution phase
                let callable = RV::Callable(RVCallable::new(fun, Datatype::Unit, Datatype::Unit));

                if let Some(Identifier { name, .. }) = name {
                    // TODO(vck): Callable shouldn't be cloned here
                    state.env.define(self.intern_string(name), callable.clone());
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
                let lower_eval = self.eval(lower, state)?;
                let upper_eval = self.eval(upper, state)?;
                let subject_eval = self.eval(subject, state)?;

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
                let root = self.look_up_variable(&head.name, e, state);

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
                let object_eval = self.eval(object, state)?;
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
                let object_eval = self.eval(object, state)?;
                if let RV::Object(mut map) = object_eval {
                    let evaluated = self.eval(value, state)?;
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
                // TODO(LYK-28)
                /*let mut planner = Planner::new(self);
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
                }*/
                Err(HaltReason::Error(
                    InterpretError::NoProgramLoaded.into(),
                ))
            }
        }
    }
}