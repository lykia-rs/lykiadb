use crate::execution::dispatching::dispatch_query_execute;
use crate::execution::global::intern_string;
use crate::execution::state::ProgramState;
use crate::interpreter::HaltReason;
use crate::interpreter::environment::{EnvironmentFrame, EnvironmentOrigin};
use crate::interpreter::error::InterpretError;
use crate::value::RV;
use crate::value::array::RVArray;
use crate::value::callable::{Function, RVCallable};
use crate::value::eval::{eval_between, eval_binary};
use crate::value::object::RVObject;
use std::sync::Arc;

use lykiadb_lang::ast::expr::{Expr, BinaryOp, TernaryOp, UnaryOp};
use lykiadb_lang::ast::{Identifier, Literal, Spanned};
use lykiadb_lang::types::Datatype;
use rustc_hash::FxHashMap;

#[derive(Clone)]
pub struct ExprEngine;

impl<'sess> ExprEngine {
    pub fn eval(
        &self,
        e: &Expr,
        state: &ProgramState<'sess>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        self.visit_expr(e, state)
    }
}

impl<'sess> ExprEngine {
    fn eval_unary(
        &self,
        operation: &UnaryOp,
        expr: &Expr,
        state: &ProgramState<'sess>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        if *operation == UnaryOp::Minus {
            if let Some(num) = self.eval(expr, state)?.to_double() {
                return Ok(RV::Double(-num));
            }
            Ok(RV::Undefined)
        } else {
            Ok(RV::Bool(!self.eval(expr, state)?.to_bool()))
        }
    }

    fn eval_binary(
        &self,
        lexpr: &Expr,
        rexpr: &Expr,
        operation: BinaryOp,
        state: &ProgramState<'sess>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        let left_eval = self.eval(lexpr, state)?;
        let right_eval = self.eval(rexpr, state)?;

        Ok(eval_binary(left_eval, right_eval, operation))
    }

    fn eval_ternary(
        &self,
        subject: &Expr,
        lower: &Expr,
        upper: &Expr,
        operation: &TernaryOp,
        state: &ProgramState<'sess>,
    ) -> Result<Option<RV<'sess>>, HaltReason<'sess>> {
        let lower_eval = self.eval(lower, state)?;
        let upper_eval = self.eval(upper, state)?;
        let subject_eval = self.eval(subject, state)?;

        if operation == &TernaryOp::Between || operation == &TernaryOp::NotBetween {
            let is_between = eval_between(&subject_eval, &lower_eval, &upper_eval);

            if let Some(is_between) = is_between {
                return match operation {
                    TernaryOp::Between => Ok(Some(RV::Bool(is_between))),
                    TernaryOp::NotBetween => Ok(Some(RV::Bool(!is_between))),
                    _ => Ok(None),
                };
            }
        }

        Ok(None)
    }

    fn is_query(&self, state: &ProgramState<'sess>) -> bool {
        state.env.origin == EnvironmentOrigin::Query
    }

    fn eval_variable(
        &self,
        name: &str,
        expr: &Expr,
        state: &ProgramState<'sess>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        let distance = state.program.as_ref().get_distance(expr);
        if let Some(unwrapped) = distance {
            EnvironmentFrame::read_at(&state.env, unwrapped, name, &intern_string(name))
        } else if state.env.origin == EnvironmentOrigin::Query {
            EnvironmentFrame::read_at(&state.env, 0, name, &intern_string(name))
        } else {
            state.root_env.read(name, &intern_string(name))
        }
    }

    fn eval_literal(
        &self,
        literal: &Literal,
        state: &ProgramState<'sess>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
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

    fn visit_expr(
        &self,
        e: &Expr,
        state: &ProgramState<'sess>,
    ) -> Result<RV<'sess>, HaltReason<'sess>> {
        match e {
            Expr::Literal { value, .. } => self.eval_literal(value, state),
            Expr::Variable { name, .. } => self.eval_variable(&name.name, e, state),
            Expr::Unary {
                operation, expr, ..
            } => self.eval_unary(operation, expr, state),
            Expr::Binary {
                operation,
                left,
                right,
                ..
            } => self.eval_binary(left, right, *operation, state),
            Expr::Ternary {
                lower,
                upper,
                subject,
                operation,
                span,
                ..
            } => self
                .eval_ternary(subject, lower, upper, operation, state)?
                .ok_or(HaltReason::Error(
                    InterpretError::InvalidRangeBoundaries { span: *span }.into(),
                )),
            Expr::Grouping { expr, .. } => self.eval(expr, state),
            Expr::Logical {
                left,
                operation,
                right,
                ..
            } => {
                let is_true = self.eval(left, state)?.to_bool();

                if (*operation == BinaryOp::Or && is_true)
                    || (*operation == BinaryOp::And && !is_true)
                {
                    return Ok(RV::Bool(is_true));
                }

                Ok(RV::Bool(self.eval(right, state)?.to_bool()))
            }
            Expr::Assignment { dst, expr, .. } => {
                let distance = state.program.as_ref().get_distance(e);

                let evaluated = self.eval(expr, state)?;
                let dst_symbol = intern_string(&dst.name);
                if let Some(distance_unv) = distance {
                    EnvironmentFrame::assign_at(
                        &state.env,
                        distance_unv,
                        &dst.name,
                        dst_symbol,
                        evaluated.clone(),
                    )
                } else {
                    state
                        .root_env
                        .assign(&dst.name, dst_symbol, evaluated.clone())
                }?;
                Ok(evaluated)
            }
            Expr::Call {
                callee, args, span, ..
            } => {
                let eval = self.eval(callee, state)?;
                if let RV::Callable(callable) = eval {
                    if self.is_query(state) && callable.is_agg() {
                        let value = self.eval_variable(&e.sign(), e, state);

                        if value.is_ok() {
                            return value;
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
                    .map(|(x, _)| intern_string(&x.name))
                    .collect();

                let fun = Function::UserDefined {
                    name: intern_string(fn_name),
                    body: Arc::clone(body),
                    parameters: param_identifiers,
                    closure: state.env.clone(),
                };

                // TODO(vck): Type evaluation should be moved to a pre-execution phase
                let callable = RV::Callable(RVCallable::new(fun, Datatype::Unit, Datatype::Unit));

                if let Some(Identifier { name, .. }) = name {
                    // TODO(vck): Callable shouldn't be cloned here
                    state.env.define(intern_string(name), callable.clone());
                }

                Ok(callable)
            }
            Expr::FieldPath {
                head,
                tail,
                span,
                id,
            } => {
                let root = self.eval_variable(&head.name, e, state);

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
            Expr::Select { span, .. }
            | Expr::Insert { span, .. }
            | Expr::Update { span, .. }
            | Expr::Delete { span, .. } => dispatch_query_execute(e, span, state),
        }
    }
}
