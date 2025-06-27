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
    visitor::{ExprReducer, ExprVisitor, ExprVisitorNode},
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

    let mut collector = AggregationCollector {
        in_call: 0,
        accumulator: vec![],
        interpreter,
    };

    let mut visitor = ExprVisitor::<Aggregation, HaltReason>::new(&mut collector);

    for projection in &core.projection {
        if let SqlProjection::Expr { expr, .. } = projection {
            let found = visitor.visit(expr)?;
            for agg in found {
                aggregates.insert(agg);
            }
        }
    }

    if let Some(expr) = &core.having {
        let found = visitor.visit(expr)?;
        for agg in found {
            aggregates.insert(agg);
        }
    }

    let mut no_dup: Vec<Aggregation> = aggregates.drain().collect();

    no_dup.sort_by_key(|a| a.to_string());

    Ok(no_dup)
}

struct AggregationCollector<'a> {
    in_call: u32,
    accumulator: Vec<Aggregation>,
    interpreter: &'a mut Interpreter,
}

impl<'a> ExprReducer<Aggregation, HaltReason> for AggregationCollector<'a> {
    fn visit(&mut self, expr: &Expr, visit: ExprVisitorNode) -> Result<bool, HaltReason> {
        if let Expr::Call { callee, args, .. } = expr {
            let callee_val = self.interpreter.eval(callee);

            if let Ok(RV::Callable(callable)) = &callee_val
                && let CallableKind::Aggregator(agg_name) = &callable.kind
            {
                match visit {
                    ExprVisitorNode::In => {
                        if self.in_call > 0 {
                            return Err(HaltReason::Error(ExecutionError::Plan(
                                PlannerError::NestedAggregationNotAllowed(expr.get_span()),
                            )));
                        }
                        self.in_call += 1;
                        self.accumulator.push(Aggregation {
                            name: agg_name.clone(),
                            args: args.clone(),
                        });
                    }
                    ExprVisitorNode::Out => {
                        self.in_call -= 1;
                    }
                }
            } else {
                return Err(callee_val.err().unwrap());
            }
        }

        Ok(true)
    }

    fn finalize(&mut self) -> Result<Vec<Aggregation>, HaltReason> {
        Ok(self.accumulator.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lykiadb_lang::ast::{
        Identifier, IdentifierKind, Span,
        expr::Expr,
        sql::{SqlProjection, SqlSelectCore},
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

        let mut collector = AggregationCollector {
            in_call: 0,
            accumulator: vec![],
            interpreter: &mut interpreter,
        };

        let mut visitor = ExprVisitor::<Aggregation, HaltReason>::new(&mut collector);

        let result = visitor.visit(&outer_avg_call);
        assert!(matches!(
            result,
            Err(HaltReason::Error(ExecutionError::Plan(
                PlannerError::NestedAggregationNotAllowed(_)
            )))
        ));
    }
}
