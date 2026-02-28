use std::collections::HashSet;

use crate::{
    error::ExecutionError,
    interpreter::{HaltReason, expr::ExprEngine},
    query::plan::{Aggregation, planner::InClause},
    value::{RV, callable::Function},
};

use lykiadb_lang::ast::{
    Spanned,
    expr::Expr,
    sql::{SqlProjection, SqlSelectCore},
    visitor::{ExprReducer, ExprVisitor, ExprVisitorNode},
};

use super::error::PlannerError;

/// Collects all the aggregates from the projection and the having clause.
/// The aggregates are stored in a HashSet to avoid duplicates and then
/// returned as a Vec<Aggregation>. For the time being, we only find
/// aggregates in the projection and the having clause.
pub fn collect_aggregates<'v>(
    core: &SqlSelectCore,
    expr_engine: &ExprEngine<'v>,
) -> Result<Vec<Aggregation<'v>>, HaltReason<'v>> {
    let mut aggregates: HashSet<Aggregation> = HashSet::new();

    let mut collector = AggregationCollector::collecting(expr_engine, InClause::Projection);

    let mut visitor = ExprVisitor::<Aggregation, HaltReason>::new(&mut collector);

    for projection in &core.projection {
        if let SqlProjection::Expr { expr, .. } = projection {
            let found = visitor.visit(expr)?;
            for agg in found {
                aggregates.insert(agg);
            }
        }
    }

    collector = AggregationCollector::collecting(expr_engine, InClause::Having);

    visitor = ExprVisitor::<Aggregation, HaltReason>::new(&mut collector);

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

pub fn prevent_aggregates_in<'v>(
    expr: &Expr,
    in_clause: InClause,
    expr_engine: &ExprEngine<'v>,
) -> Result<Vec<Aggregation<'v>>, HaltReason<'v>> {
    let _ = expr_engine;
    let mut collector = AggregationCollector::preventing(expr_engine, in_clause);

    let mut visitor = ExprVisitor::<Aggregation, HaltReason>::new(&mut collector);

    let aggregates = visitor.visit(expr)?;

    Ok(aggregates)
}

struct AggregationCollector<'a, 'v> {
    in_call: u32,
    accumulator: Vec<Aggregation<'v>>,
    expr_engine: &'a ExprEngine<'v>,
    is_preventing: bool,
    in_clause: InClause,
}

impl<'a, 'v> AggregationCollector<'a, 'v> {
    fn preventing(
        expr_engine: &'a ExprEngine<'v>,
        in_clause: InClause,
    ) -> AggregationCollector<'a, 'v> {
        AggregationCollector {
            in_call: 0,
            accumulator: vec![],
            expr_engine,
            is_preventing: true,
            in_clause,
        }
    }

    fn collecting(
        expr_engine: &'a ExprEngine<'v>,
        in_clause: InClause,
    ) -> AggregationCollector<'a, 'v> {
        AggregationCollector {
            in_call: 0,
            accumulator: vec![],
            expr_engine,
            is_preventing: false,
            in_clause,
        }
    }
}

impl<'a, 'v> ExprReducer<Aggregation<'v>, HaltReason<'v>> for AggregationCollector<'a, 'v> {
    fn visit(&mut self, expr: &Expr, visit: ExprVisitorNode) -> Result<bool, HaltReason<'v>> {
        if let Expr::Call { callee, args, .. } = expr {
            let callee_val = self.expr_engine.eval(callee);

            if let Ok(RV::Callable(callable)) = &callee_val
                && let Function::Agg {
                    function: factory,
                    name: agg_name,
                } = &callable.function.as_ref()
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
                        self.accumulator
                            .push(Aggregation::new(agg_name, factory, args, expr));
                    }
                    ExprVisitorNode::Out => {
                        self.in_call -= 1;
                    }
                }
            }
        }

        Ok(true)
    }

    fn finalize(&mut self) -> Result<Vec<Aggregation<'v>>, HaltReason<'v>> {
        Ok(self.accumulator.drain(..).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::tests::create_test_interpreter;

    use super::*;
    use lykiadb_lang::ast::{
        Identifier, IdentifierKind, Span,
        expr::Expr,
        sql::{SqlProjection, SqlSelectCore},
    };

    #[test]
    fn test_collect_aggregates_simple_projection() -> Result<(), HaltReason<'static>> {
        let mut interpreter = create_test_interpreter(None);

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
            span: Span::default(),
        };

        let result = collect_aggregates(&core, interpreter.get_expr_engine())?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "avg");

        Ok(())
    }

    #[test]
    fn test_collect_aggregates_having_clause() -> Result<(), HaltReason<'static>> {
        let mut interpreter = create_test_interpreter(None);

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
            span: Span::default(),
        };

        let result = collect_aggregates(&core, interpreter.get_expr_engine())?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "avg");

        Ok(())
    }

    #[test]
    fn test_nested_aggregates_not_allowed() {
        let mut interpreter = create_test_interpreter(None);

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
            AggregationCollector::collecting(interpreter.get_expr_engine(), InClause::Projection);

        let mut visitor = ExprVisitor::<Aggregation, HaltReason>::new(&mut collector);

        let result = visitor.visit(&outer_avg_call);
        assert!(matches!(
            result,
            Err(HaltReason::Error(ExecutionError::Plan(
                PlannerError::NestedAggregationNotAllowed(_)
            )))
        ));
    }

    #[test]
    fn test_aggregation_should_be_drained_after_each_visit() -> Result<(), HaltReason<'static>> {
        let mut interpreter = create_test_interpreter(None);

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
            AggregationCollector::collecting(interpreter.get_expr_engine(), InClause::Projection);

        let mut visitor = ExprVisitor::<Aggregation, HaltReason>::new(&mut collector);

        let result1 = visitor.visit(&avg_call)?;
        assert_eq!(result1.len(), 1);
        assert_eq!(result1[0].name, "avg");

        let result2 = visitor.visit(&avg_call)?;
        assert_eq!(result2.len(), 1);
        assert_eq!(result2[0].name, "avg");

        Ok(())
    }
}
