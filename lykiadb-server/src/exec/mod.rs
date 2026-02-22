use lykiadb_lang::ast::sql::SqlProjection;

use crate::{
    engine::{
        error::ExecutionError,
        interpreter::{HaltReason, Interpreter},
    },
    exec::aggregation::Grouper,
    global::GLOBAL_INTERNER,
    plan::{IntermediateExpr, Node, Plan},
    value::{
        RV, iterator::{ExecutionRow, RVs}
    },
};

pub mod aggregation;

pub struct PlanExecutor<'a, 'v> {
    interpreter: &'a mut Interpreter<'v>,
}

impl<'a, 'v> PlanExecutor<'a, 'v> {
    pub fn new(interpreter: &'a mut Interpreter<'v>) -> PlanExecutor<'a, 'v> {
        PlanExecutor { interpreter }
    }

    pub fn execute_plan(&mut self, plan: Plan<'v>) -> Result<RVs<'v>, ExecutionError> {
        // Placeholder for plan execution logic
        match plan {
            Plan::Select(root) => {
                // Execute scan plan
                self.execute_node(root.clone())
            }
        }
    }

    pub fn execute_node(&mut self, node: Node<'v>) -> Result<RVs<'v>, ExecutionError> {
        match node {
            Node::Subquery { source, alias } => {
                let cursor = self.execute_node(*source)?;

                let iter = cursor.map(move |row: ExecutionRow| {
                    let mut upstream = ExecutionRow::new();
                    let key = alias.to_string();
                    upstream.insert(GLOBAL_INTERNER.intern(&key), row.as_value());
                    upstream
                });

                Ok(Box::from(iter))
            }
            Node::Offset { source, offset } => {
                Ok(Box::from(self.execute_node(*source)?.skip(offset)))
            }
            Node::Limit { source, limit } => Ok(Box::from(self.execute_node(*source)?.take(limit))),
            Node::Filter {
                source,
                predicate,
                subqueries,
            } => {
                match predicate {
                    IntermediateExpr::Constant(ct) => {
                        // TODO(vck): Maybe we can deal with this at compile time?
                        if ct.as_bool() {
                            let cursor = self.execute_node(*source)?;
                            Ok(cursor)
                        } else {
                            let empty_iter = Vec::<ExecutionRow<'v>>::new().into_iter();
                            Ok(Box::from(empty_iter) as RVs)
                        }
                    }
                    IntermediateExpr::Expr { expr } => {
                        let cursor = self.execute_node(*source)?;

                        let mut inter_fork = self.interpreter.clone();

                        let iter = cursor.filter_map(move |row: ExecutionRow| {
                            let evaluated = inter_fork.eval_with_row(&expr, &row);
                            if let Ok(value) = evaluated
                                && value.as_bool()
                            {
                                Some(row)
                            } else {
                                None
                            }
                        });

                        Ok(Box::from(iter))
                    }
                }
            }
            Node::Projection { source, fields } => {
                let cursor = self.execute_node(*source)?;

                let mut inter_fork = self.interpreter.clone();

                let iter = cursor.map(move |downstream: ExecutionRow| {
                    let mut upstream = ExecutionRow::new();

                    for field in &fields {
                        match field {
                            SqlProjection::All { collection } => {
                                if collection.is_none() {
                                    downstream.copy_to(&mut upstream);
                                } else {
                                    let projected_key = collection.as_ref().unwrap().to_string();
                                    let interned_key = GLOBAL_INTERNER.intern(&projected_key);
                                    let value = &downstream.get(&interned_key);
                                    upstream.insert(interned_key, value.unwrap().clone());
                                }
                            }
                            SqlProjection::Expr { expr, alias } => {
                                let evaluated = inter_fork.eval_with_row(expr, &downstream);
                                let value = match evaluated {
                                    Ok(v) => v,
                                    Err(_) => RV::Undefined,
                                };
                                let key = alias
                                    .as_ref()
                                    .map(|a| a.to_string())
                                    .unwrap_or(expr.to_string());

                                upstream.insert(GLOBAL_INTERNER.intern(&key), value);
                            }
                        }
                    }

                    upstream
                });

                Ok(Box::from(iter))
            }
            Node::EvalScan { source, filter } => match self.interpreter.eval(&source.expr) {
                Err(HaltReason::Error(err)) => Err(err),
                Err(HaltReason::Return(value)) | Ok(value) => {
                    let alias = source.alias.to_owned();

                    let sym_alias = GLOBAL_INTERNER.intern(&alias.to_string());

                    let mapper = move |v: RV<'v>| {
                        let mut env = ExecutionRow::new();
                        env.insert(sym_alias, v.clone());
                        env
                    };

                    let iter = match value {
                        RV::Array(arr) => {
                            let c = arr.collect();
                            c.into_iter().map(mapper)
                        }
                        _ => vec![value]
                            .into_iter()
                            .collect::<Vec<_>>()
                            .into_iter()
                            .map(mapper),
                    };

                    Ok(Box::from(iter))
                }
            },
            Node::Aggregate {
                source,
                group_by,
                aggregates,
            } => {
                let inter_fork = self.interpreter.clone();

                let mut grouper = Grouper::new(group_by, aggregates, inter_fork);

                let cursor = self.execute_node(*source)?;

                for row in cursor {
                    if let Err(e) = grouper.row(row) {
                        if let HaltReason::Error(err) = e {
                            return Err(err);
                        }
                    }
                }

                let rows = grouper.finalize();

                Ok(Box::from(rows.into_iter()))
            }
            Node::Join {
                left,
                join_type,
                right,
                constraint,
            } => todo!(),
            Node::Order { source, key } => todo!(),
            Node::Scan { source, filter } => todo!(),
            Node::Compound {
                source,
                operator,
                right,
            } => todo!(),
            Node::Nothing => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::interpreter::tests::create_test_interpreter;
    use crate::plan::IntermediateExpr;
    use crate::value::RV;
    use lykiadb_lang::ast::{Identifier, IdentifierKind, Literal, expr::Expr, sql::SqlProjection};
    use std::sync::Arc;

    fn create_test_executor() -> PlanExecutor<'static, 'static> {
        let interpreter = Box::leak(Box::from(create_test_interpreter(None)));

        PlanExecutor::new(interpreter)
    }

    fn create_test_identifier(name: &str) -> Identifier {
        Identifier::new(name, IdentifierKind::Variable)
    }

    fn create_literal_expr(literal: Literal) -> Expr {
        Expr::Literal {
            value: literal,
            raw: String::new(),
            span: lykiadb_lang::ast::Span::default(),
            id: 0,
        }
    }

    #[test]
    fn test_execute_plan_select() {
        let mut executor = create_test_executor();

        // Create a simple EvalScan node
        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(create_literal_expr(Literal::Array(vec![]))),
            alias: create_test_identifier("test_alias"),
        };

        let node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        let plan = Plan::Select(node);

        let result = executor.execute_plan(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_node_evalscan_with_array() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create an EvalScan node with an array literal
        let array_expr = create_literal_expr(Literal::Array(vec![
            create_literal_expr(Literal::Num(1.0)),
            create_literal_expr(Literal::Num(2.0)),
            create_literal_expr(Literal::Num(3.0)),
        ]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        let result = executor.execute_node(node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 3);

        Ok(())
    }

    #[test]
    fn test_execute_node_evalscan_with_single_value() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create an EvalScan node with a single value
        let value_expr = create_literal_expr(Literal::Str(Arc::new("test_value".to_string())));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(value_expr),
            alias: create_test_identifier("single"),
        };

        let node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        let result = executor.execute_node(node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 1);

        let symbol = GLOBAL_INTERNER.intern("single");

        assert!(matches!(rows[0].get(&symbol), Some(RV::Str(_))));

        Ok(())
    }

    #[test]
    fn test_execute_node_limit() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create source node (EvalScan with array)
        let array_expr = create_literal_expr(Literal::Array(vec![
            create_literal_expr(Literal::Num(1.0)),
            create_literal_expr(Literal::Num(2.0)),
            create_literal_expr(Literal::Num(3.0)),
            create_literal_expr(Literal::Num(4.0)),
            create_literal_expr(Literal::Num(5.0)),
        ]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let source_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Create limit node
        let limit_node = Node::Limit {
            source: Box::new(source_node),
            limit: 2,
        };

        let result = executor.execute_node(limit_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 2);
        let symbol = GLOBAL_INTERNER.intern("numbers");
        assert_eq!(rows[0].get(&symbol), Some(&RV::Double(1.0)));
        assert_eq!(rows[1].get(&symbol), Some(&RV::Double(2.0)));

        Ok(())
    }

    #[test]
    fn test_execute_node_offset() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create source node (EvalScan with array)
        let array_expr = create_literal_expr(Literal::Array(vec![
            create_literal_expr(Literal::Num(1.0)),
            create_literal_expr(Literal::Num(2.0)),
            create_literal_expr(Literal::Num(3.0)),
            create_literal_expr(Literal::Num(4.0)),
            create_literal_expr(Literal::Num(5.0)),
        ]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let source_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Create offset node
        let offset_node = Node::Offset {
            source: Box::new(source_node),
            offset: 2,
        };

        let result = executor.execute_node(offset_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 3);
        let symbol = GLOBAL_INTERNER.intern("numbers");
        assert_eq!(rows[0].get(&symbol), Some(&RV::Double(3.0)));
        assert_eq!(rows[1].get(&symbol), Some(&RV::Double(4.0)));
        assert_eq!(rows[2].get(&symbol), Some(&RV::Double(5.0)));

        Ok(())
    }

    #[test]
    fn test_execute_node_subquery() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create source node (EvalScan with array)
        let array_expr = create_literal_expr(Literal::Array(vec![
            create_literal_expr(Literal::Num(1.0)),
            create_literal_expr(Literal::Num(2.0)),
        ]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let source_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Create subquery node
        let subquery_node = Node::Subquery {
            source: Box::new(source_node),
            alias: create_test_identifier("sub"),
        };

        let result = executor.execute_node(subquery_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 2);

        // Check that each row has the alias key
        let symbol = GLOBAL_INTERNER.intern("sub");

        assert_eq!(
            rows[0]
                .get(&symbol)
                .unwrap()
                .if_object()
                .unwrap()
                .get("numbers")
                .unwrap(),
            RV::Double(1.0)
        );
        assert_eq!(
            rows[1]
                .get(&symbol)
                .unwrap()
                .if_object()
                .unwrap()
                .get("numbers")
                .unwrap(),
            RV::Double(2.0)
        );

        Ok(())
    }

    #[test]
    fn test_execute_node_filter_constant_true() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create source node
        let array_expr = create_literal_expr(Literal::Array(vec![
            create_literal_expr(Literal::Num(1.0)),
            create_literal_expr(Literal::Num(2.0)),
        ]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let source_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Create filter node with constant true
        let filter_node = Node::Filter {
            source: Box::new(source_node),
            predicate: IntermediateExpr::Constant(RV::Bool(true)),
            subqueries: vec![],
        };

        let result = executor.execute_node(filter_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 2);

        Ok(())
    }

    #[test]
    fn test_execute_node_filter_constant_false() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create source node
        let array_expr = create_literal_expr(Literal::Array(vec![
            create_literal_expr(Literal::Num(1.0)),
            create_literal_expr(Literal::Num(2.0)),
        ]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let source_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Create filter node with constant false
        let filter_node = Node::Filter {
            source: Box::new(source_node),
            predicate: IntermediateExpr::Constant(RV::Bool(false)),
            subqueries: vec![],
        };

        let result = executor.execute_node(filter_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 0);

        Ok(())
    }

    #[test]
    fn test_execute_node_projection_all() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create source node
        let array_expr = create_literal_expr(Literal::Array(vec![
            create_literal_expr(Literal::Num(1.0)),
            create_literal_expr(Literal::Num(2.0)),
        ]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let source_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Create projection node with All
        let projection_node = Node::Projection {
            source: Box::new(source_node),
            fields: vec![SqlProjection::All { collection: None }],
        };

        let result = executor.execute_node(projection_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 2);

        Ok(())
    }

    #[test]
    fn test_execute_node_projection_expr() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create source node
        let array_expr =
            create_literal_expr(Literal::Array(vec![create_literal_expr(Literal::Num(1.0))]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let source_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Create projection node with expression
        let projection_node = Node::Projection {
            source: Box::new(source_node),
            fields: vec![SqlProjection::Expr {
                expr: Box::new(create_literal_expr(Literal::Str(Arc::new(
                    "projected_value".to_string(),
                )))),
                alias: Some(create_test_identifier("projected")),
            }],
        };

        let result = executor.execute_node(projection_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 1);

        // Check the projected value
        let symbol = GLOBAL_INTERNER.intern("projected");

        assert!(
            matches!(rows[0].get(&symbol), Some(RV::Str(s)) if s.as_str() == "projected_value")
        );

        Ok(())
    }

    #[test]
    fn test_execute_node_evalscan_empty_array() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create an EvalScan node with an empty array
        let array_expr = create_literal_expr(Literal::Array(vec![]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("empty"),
        };

        let node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        let result = executor.execute_node(node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 0);

        Ok(())
    }

    #[test]
    fn test_execute_node_limit_larger_than_available() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create source node with 2 items
        let array_expr = create_literal_expr(Literal::Array(vec![
            create_literal_expr(Literal::Num(1.0)),
            create_literal_expr(Literal::Num(2.0)),
        ]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let source_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Create limit node with limit larger than available items
        let limit_node = Node::Limit {
            source: Box::new(source_node),
            limit: 10,
        };

        let result = executor.execute_node(limit_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        // Should return all available items
        assert_eq!(rows.len(), 2);

        Ok(())
    }

    #[test]
    fn test_execute_node_limit_zero() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create source node
        let array_expr = create_literal_expr(Literal::Array(vec![
            create_literal_expr(Literal::Num(1.0)),
            create_literal_expr(Literal::Num(2.0)),
        ]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let source_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Create limit node with zero limit
        let limit_node = Node::Limit {
            source: Box::new(source_node),
            limit: 0,
        };

        let result = executor.execute_node(limit_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 0);

        Ok(())
    }

    #[test]
    fn test_execute_node_offset_larger_than_available() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create source node with 2 items
        let array_expr = create_literal_expr(Literal::Array(vec![
            create_literal_expr(Literal::Num(1.0)),
            create_literal_expr(Literal::Num(2.0)),
        ]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let source_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Create offset node with offset larger than available items
        let offset_node = Node::Offset {
            source: Box::new(source_node),
            offset: 10,
        };

        let result = executor.execute_node(offset_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        // Should return empty result
        assert_eq!(rows.len(), 0);

        Ok(())
    }

    #[test]
    fn test_execute_node_projection_all_with_collection() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create source node
        let array_expr =
            create_literal_expr(Literal::Array(vec![create_literal_expr(Literal::Num(1.0))]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("test_collection"),
        };

        let source_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Create projection node with All from specific collection
        let projection_node = Node::Projection {
            source: Box::new(source_node),
            fields: vec![SqlProjection::All {
                collection: Some(create_test_identifier("test_collection")),
            }],
        };

        let result = executor.execute_node(projection_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 1);

        Ok(())
    }

    #[test]
    fn test_complex_plan_execution() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create a complex plan with multiple nodes: EvalScan -> Filter -> Limit
        let array_expr = create_literal_expr(Literal::Array(vec![
            create_literal_expr(Literal::Num(1.0)),
            create_literal_expr(Literal::Num(2.0)),
            create_literal_expr(Literal::Num(3.0)),
            create_literal_expr(Literal::Num(4.0)),
            create_literal_expr(Literal::Num(5.0)),
        ]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let scan_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Add a filter that always returns true (constant)
        let filter_node = Node::Filter {
            source: Box::new(scan_node),
            predicate: IntermediateExpr::Constant(RV::Bool(true)),
            subqueries: vec![],
        };

        // Add a limit
        let limit_node = Node::Limit {
            source: Box::new(filter_node),
            limit: 3,
        };

        // Create the final plan
        let plan = Plan::Select(limit_node);

        let result = executor.execute_plan(plan);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        // Should have 3 rows due to limit
        assert_eq!(rows.len(), 3);

        Ok(())
    }

    #[test]
    fn test_complex_plan_with_offset_and_projection() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create a plan with: EvalScan -> Offset -> Projection -> Subquery
        let array_expr = create_literal_expr(Literal::Array(vec![
            create_literal_expr(Literal::Num(1.0)),
            create_literal_expr(Literal::Num(2.0)),
            create_literal_expr(Literal::Num(3.0)),
        ]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("values"),
        };

        let scan_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Add offset to skip first element
        let offset_node = Node::Offset {
            source: Box::new(scan_node),
            offset: 1,
        };

        // Add projection
        let projection_node = Node::Projection {
            source: Box::new(offset_node),
            fields: vec![SqlProjection::All { collection: None }],
        };

        // Wrap in subquery
        let subquery_node = Node::Subquery {
            source: Box::new(projection_node),
            alias: create_test_identifier("wrapped"),
        };

        let result = executor.execute_node(subquery_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        // Should have 2 rows (3 original - 1 offset)
        assert_eq!(rows.len(), 2);

        // Check that subquery alias is present
        let symbol = GLOBAL_INTERNER.intern("wrapped");
        for row in &rows {
            assert!(row.get(&symbol).is_some());
        }

        Ok(())
    }

    #[test]
    fn test_evalscan_with_different_value_types() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Test with boolean value
        let bool_expr = create_literal_expr(Literal::Bool(true));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(bool_expr),
            alias: create_test_identifier("bool_val"),
        };

        let node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        let result = executor.execute_node(node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 1);

        let symbol = GLOBAL_INTERNER.intern("bool_val");

        assert!(matches!(rows[0].get(&symbol), Some(RV::Bool(true))));

        Ok(())
    }

    #[test]
    fn test_projection_expr_with_undefined_result() -> Result<(), ExecutionError> {
        let mut executor = create_test_executor();

        // Create source node with one item
        let array_expr =
            create_literal_expr(Literal::Array(vec![create_literal_expr(Literal::Num(1.0))]));

        let eval_source = lykiadb_lang::ast::sql::SqlExpressionSource {
            expr: Box::new(array_expr),
            alias: create_test_identifier("numbers"),
        };

        let source_node = Node::EvalScan {
            source: eval_source,
            filter: None,
        };

        // Create projection with undefined literal
        let projection_node = Node::Projection {
            source: Box::new(source_node),
            fields: vec![SqlProjection::Expr {
                expr: Box::new(create_literal_expr(Literal::Undefined)),
                alias: Some(create_test_identifier("undef")),
            }],
        };

        let result = executor.execute_node(projection_node);
        assert!(result.is_ok());

        let iterator = result?;
        let rows: Vec<ExecutionRow> = iterator.collect();
        assert_eq!(rows.len(), 1);

        let symbol = GLOBAL_INTERNER.intern("undef");
        assert!(matches!(rows[0].get(&symbol), Some(RV::Undefined)));

        Ok(())
    }
}
