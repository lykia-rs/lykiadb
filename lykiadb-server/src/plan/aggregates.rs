use std::collections::HashSet;

use crate::{
    engine::{
        error::ExecutionError,
        interpreter::{Aggregation, HaltReason, Interpreter},
    },
    value::{RV, callable::CallableKind},
};

use lykiadb_lang::ast::{
    Spanned,
    expr::Expr,
    sql::{SqlProjection, SqlSelectCore},
};

use super::PlannerError;

// Collects all the aggregates from the projection and the having clause.
// The aggregates are stored in a HashSet to avoid duplicates and then
// returned as a Vec<Aggregation>. For the time being, we only find
// aggregates in the projection and the having clause.
pub fn collect_aggregates<'a>(
    core: &SqlSelectCore,
    interpreter: &'a mut Interpreter,
) -> Result<Vec<Aggregation>, HaltReason> {
    let mut aggregates: HashSet<Aggregation> = HashSet::new();

    for projection in &core.projection {
        if let SqlProjection::Expr { expr, .. } = projection {
            let found = collect_aggregates_from_expr(expr, interpreter)?;
            for agg in found {
                aggregates.insert(agg);
            }
        }
    }

    if let Some(expr) = &core.having {
        let found = collect_aggregates_from_expr(expr, interpreter)?;
        for agg in found {
            aggregates.insert(agg);
        }
    }

    let no_dup = aggregates.drain().collect();

    Ok(no_dup)
}

fn collect_aggregates_from_expr<'a>(
    expr: &Expr,
    interpreter: &'a mut Interpreter,
) -> Result<Vec<Aggregation>, HaltReason> {
    match expr {
        Expr::Select { .. }
        | Expr::Insert { .. }
        | Expr::Delete { .. }
        | Expr::Update { .. }
        | Expr::Variable { .. }
        | Expr::Literal { .. }
        | Expr::FieldPath { .. }
        | Expr::Function { .. }
        | Expr::Set { .. } => Ok(vec![]),
        //
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            let rleft = collect_aggregates_from_expr(left, interpreter);
            let rright = collect_aggregates_from_expr(right, interpreter);

            let mut result = vec![];
            if let Ok(v) = rleft {
                result.extend(v);
            }
            if let Ok(v) = rright {
                result.extend(v);
            }
            Ok(result)
        }
        //
        Expr::Grouping { expr, .. } | Expr::Unary { expr, .. } | Expr::Assignment { expr, .. } => {
            let r = collect_aggregates_from_expr(expr, interpreter);
            if let Ok(v) = r {
                return Ok(v);
            }
            Ok(vec![])
        }
        //
        Expr::Call { callee, args, .. } => {
            let mut result: Vec<Aggregation> = vec![];

            let callee_val = interpreter.eval(callee);

            if let Ok(RV::Callable(callable)) = &callee_val {
                if let CallableKind::Aggregator(agg_name) = &callable.kind {
                    result.push(Aggregation {
                        name: agg_name.clone(),
                        args: args.clone(),
                    });
                }
            }
            if callee_val.is_err() {
                return Err(callee_val.err().unwrap());
            }
            let rargs: Vec<Aggregation> = args
                .iter()
                .map(|x| collect_aggregates_from_expr(x, interpreter))
                .flat_map(|x| x.unwrap())
                .collect();

            if !rargs.is_empty() {
                return Err(HaltReason::Error(ExecutionError::Plan(
                    PlannerError::NestedAggregationNotAllowed(expr.get_span()),
                )));
            }

            Ok(result)
        }
        Expr::Between {
            lower,
            upper,
            subject,
            ..
        } => {
            let rlower = collect_aggregates_from_expr(lower, interpreter);
            let rupper = collect_aggregates_from_expr(upper, interpreter);
            let rsubject = collect_aggregates_from_expr(subject, interpreter);

            let mut result = vec![];
            if let Ok(v) = rlower {
                result.extend(v);
            }
            if let Ok(v) = rupper {
                result.extend(v);
            }
            if let Ok(v) = rsubject {
                result.extend(v);
            }
            Ok(result)
        }
        Expr::Get { object, .. } => {
            let robject = collect_aggregates_from_expr(object, interpreter);
            if let Ok(v) = robject {
                return Ok(v);
            }
            Ok(vec![])
        }
    }
}
