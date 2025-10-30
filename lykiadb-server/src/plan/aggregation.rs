use std::collections::HashSet;

use crate::{
    engine::{
        error::ExecutionError,
        interpreter::{Aggregation, HaltReason, Interpreter},
    },
    plan::planner::InClause,
    value::{StdVal, Value, callable::CallableKind},
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
pub fn collect_aggregates<V: Value>(
    core: &SqlSelectCore,
    interpreter: &mut Interpreter<V>,
) -> Result<Vec<Aggregation>, HaltReason<V>> {
    let mut aggregates: HashSet<Aggregation> = HashSet::new();

    let mut collector = AggregationCollector::collecting(interpreter, InClause::Projection);

    let mut visitor = ExprVisitor::<Aggregation, HaltReason<V>>::new(&mut collector);

    for projection in &core.projection {
        if let SqlProjection::Expr { expr, .. } = projection {
            let found = visitor.visit(expr)?;
            for agg in found {
                aggregates.insert(agg);
            }
        }
    }

    collector = AggregationCollector::collecting(interpreter, InClause::Having);

    visitor = ExprVisitor::<Aggregation, HaltReason<V>>::new(&mut collector);

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

pub fn prevent_aggregates_in<V: Value>(
    expr: &Expr,
    in_clause: InClause,
    interpreter: &mut Interpreter<V>,
) -> Result<Vec<Aggregation>, HaltReason<V>> {
    let mut collector = AggregationCollector::preventing(interpreter, in_clause);

    let mut visitor = ExprVisitor::<Aggregation, HaltReason<V>>::new(&mut collector);

    let aggregates = visitor.visit(expr)?;

    Ok(aggregates)
}

struct AggregationCollector<'a, V: Value> {
    in_call: u32,
    accumulator: Vec<Aggregation>,
    interpreter: &'a mut Interpreter<V>,
    is_preventing: bool,
    in_clause: InClause,
}

impl<'a, V: Value> AggregationCollector<'a, V> {
    fn preventing(interpreter: &'a mut Interpreter<V>, in_clause: InClause) -> AggregationCollector<'a, V> {
        AggregationCollector {
            in_call: 0,
            accumulator: vec![],
            interpreter,
            is_preventing: true,
            in_clause,
        }
    }

    fn collecting(interpreter: &'a mut Interpreter<V>, in_clause: InClause) -> AggregationCollector<'a, V> {
        AggregationCollector {
            in_call: 0,
            accumulator: vec![],
            interpreter,
            is_preventing: false,
            in_clause,
        }
    }
}

impl<'a, V: Value> ExprReducer<Aggregation, HaltReason<V>> for AggregationCollector<'a, V> {
    fn visit(&mut self, expr: &Expr, visit: ExprVisitorNode) -> Result<bool, HaltReason<V>> {
        if let Expr::Call { callee, args, .. } = expr {
            let callee_val = self.interpreter.eval(callee);

            if let Some(callable) = callee_val.unwrap_or(V::undefined()).as_callable()
                && let CallableKind::Aggregator(agg_name) = &callable.kind
            {
                if self.is_preventing {
                    return Err(HaltReason::Error(ExecutionError::Plan(
                        PlannerError::AggregationNotAllowed(
                            expr.get_span(),
                            self.in_clause.to_string(),
                        ),
                    )));
                }

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
            }
        }

        Ok(true)
    }

    fn finalize(&mut self) -> Result<Vec<Aggregation>, HaltReason<V>> {
        Ok(self.accumulator.drain(..).collect())
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

    fn create_test_interpreter() -> Interpreter<StdVal> {
        Interpreter::<StdVal>::new(None, true)
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

        let mut collector =
            AggregationCollector::collecting(&mut interpreter, InClause::Projection);

        let mut visitor = ExprVisitor::<Aggregation, HaltReason<StdVal>>::new(&mut collector);

        let result = visitor.visit(&outer_avg_call);
        assert!(matches!(
            result,
            Err(HaltReason::Error(ExecutionError::Plan(
                PlannerError::NestedAggregationNotAllowed(_)
            )))
        ));
    }

    #[test]
    fn test_aggregation_should_be_drained_after_each_visit() {
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

        let mut collector =
            AggregationCollector::collecting(&mut interpreter, InClause::Projection);

        let mut visitor = ExprVisitor::<Aggregation, HaltReason<StdVal>>::new(&mut collector);

        let result1 = visitor.visit(&avg_call).unwrap();
        assert_eq!(result1.len(), 1);
        assert_eq!(result1[0].name, "avg");

        let result2 = visitor.visit(&avg_call).unwrap();
        assert_eq!(result2.len(), 1);
        assert_eq!(result2[0].name, "avg");
    }
}
