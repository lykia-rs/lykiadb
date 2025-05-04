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

/// Collects all the aggregates from the projection and the having clause.
/// The aggregates are stored in a HashSet to avoid duplicates and then
/// returned as a Vec<Aggregation>. For the time being, we only find
/// aggregates in the projection and the having clause.
pub fn collect_aggregates(
    core: &SqlSelectCore,
    interpreter: &mut Interpreter,
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

    let mut no_dup: Vec<Aggregation> = aggregates.drain().collect();

    no_dup.sort_by_key(|a| a.to_string());

    Ok(no_dup)
}

fn collect_aggregates_from_expr(
    expr: &Expr,
    interpreter: &mut Interpreter,
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

#[cfg(test)]
mod tests {
    use super::*;
    use lykiadb_lang::ast::{
        expr::Expr, sql::{SqlProjection, SqlSelectCore}, Identifier, IdentifierKind, Span
    };

    fn create_test_interpreter() -> Interpreter {
        Interpreter::new(None, true)
    }

    #[test]
    fn test_collect_aggregates_simple_projection() {
        let mut interpreter = create_test_interpreter();

        let avg_call = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("avg", IdentifierKind::Symbol),
                span: Span::default(),
                id: 0,
            }),
            args: vec![],
            span: Span::default(),
            id: 0,
        };

        let core = SqlSelectCore {
            projection: vec![SqlProjection::Expr {
                expr: Box::new(avg_call),
                alias: None,
            }],
            from: None,
            group_by: None,
            having: None,
            distinct: lykiadb_lang::ast::sql::SqlDistinct::All,
            r#where: None,
            compound: None,
        };

        let result = collect_aggregates(&core, &mut interpreter).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "avg");
    }

    #[test]
    fn test_collect_aggregates_having_clause() {
        let mut interpreter = create_test_interpreter();

        let avg_call = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("avg", IdentifierKind::Symbol),
                span: Span::default(),
                id: 0,
            }),
            args: vec![],
            span: Span::default(),
            id: 0,
        };

        let core = SqlSelectCore {
            projection: vec![],
            from: None,
            group_by: None,
            distinct: lykiadb_lang::ast::sql::SqlDistinct::All,
            r#where: None,
            compound: None,
            having: Some(Box::new(avg_call)),
        };

        let result = collect_aggregates(&core, &mut interpreter).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "avg");
    }

    #[test]
    fn test_nested_aggregates_not_allowed() {
        let mut interpreter = create_test_interpreter();

        let avg_call = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("avg", IdentifierKind::Symbol),
                span: Span::default(),
                id: 0,
            }),
            args: vec![],
            span: Span::default(),
            id: 0,
        };

        let outer_avg_call = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("avg", IdentifierKind::Symbol),
                span: Span::default(),
                id: 0,
            }),
            args: vec![avg_call],
            span: Span::default(),
            id: 0,
        };

        let result = collect_aggregates_from_expr(&outer_avg_call, &mut interpreter);
        assert!(matches!(
            result,
            Err(HaltReason::Error(ExecutionError::Plan(
                PlannerError::NestedAggregationNotAllowed(_)
            )))
        ));
    }
}
