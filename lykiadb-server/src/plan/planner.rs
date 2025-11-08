use std::fmt::Display;

use crate::{
    engine::{
        error::ExecutionError,
        interpreter::{HaltReason, Interpreter},
    },
    plan::{PlannerError, aggregation::prevent_aggregates_in},
    value::RV,
};

use lykiadb_lang::ast::{
    Spanned,
    expr::Expr,
    sql::{SqlProjection, SqlSelect, SqlSelectCore},
    visitor::{ExprVisitor, VisitorMut},
};

use super::{
    IntermediateExpr, Node, Plan, aggregation::collect_aggregates, expr::SqlExprReducer,
    from::build_from, scope::Scope,
};

#[derive(Debug)]
pub enum InClause {
    Where,
    Projection,
    Having,
    GroupBy,
    OrderBy,
    JoinOn,
}

impl Display for InClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InClause::Where => write!(f, "WHERE"),
            InClause::Projection => write!(f, "SELECT"),
            InClause::Having => write!(f, "HAVING"),
            InClause::GroupBy => write!(f, "GROUP BY"),
            InClause::OrderBy => write!(f, "ORDER BY"),
            InClause::JoinOn => write!(f, "JOIN ON"),
        }
    }
}

pub struct Planner<'a> {
    interpreter: &'a mut Interpreter,
}

impl<'a> Planner<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Planner<'a> {
        Planner { interpreter }
    }

    pub fn build(&mut self, expr: &Expr) -> Result<Plan, HaltReason> {
        match expr {
            Expr::Select { query, .. } => {
                let plan = Plan::Select(self.build_select(query)?);
                Ok(plan)
            }
            _ => panic!("Bummer."),
        }
    }

    fn eval_constant(&mut self, expr: &Expr) -> Result<RV, HaltReason> {
        self.interpreter.visit_expr(expr)
    }

    pub fn build_expr(
        &mut self,
        expr: &Expr,
        in_clause: InClause,
        scope: &mut Scope,
        allow_subqueries: bool,
        allow_aggregates: bool,
    ) -> Result<(IntermediateExpr, Vec<Node>), HaltReason> {
        if !allow_aggregates {
            prevent_aggregates_in(expr, in_clause, self.interpreter)?;
        }

        let mut reducer: SqlExprReducer = SqlExprReducer::new(
            // self,
            allow_subqueries,
            scope,
        );

        let mut visitor = ExprVisitor::<SqlSelect, HaltReason>::new(&mut reducer);

        let selects = visitor.visit(expr)?;

        let subqueries = selects
            .into_iter()
            .map(|select| self.build_select(&select))
            .collect::<Result<Vec<Node>, HaltReason>>()?;

        Ok((
            IntermediateExpr::Expr {
                expr: Box::new(expr.clone()),
            },
            subqueries,
        ))
    }
}

// Select planner
impl<'a> Planner<'a> {
    /*

    The data flow we built using SqlSelectCore is as follows:

    +--------+      +---------+      +-----------+      +------------+      +-----------------------+
    | Source | ---> | Filter  | ---> | Aggregate | ---> | Projection | ---> | Filter                |
    | (req.) |      | (optl.) |      | (optl.)   |      | (req.)     |      | (for post projection) |
    +--------+      +---------+      +-----------+      +------------+      +-----------------------+
    */

    // The end result is a computation graph, that can be easily combined with
    // other computation graphs. A typical example is a compound query, where
    // the result of one query is used as a source for another query. The data
    // flow is as follows:

    /*
    +---------------+             +---------------+               +---------------+
    | SqlSelectCore | ----------> | SqlSelectCore | ------------> | SqlSelectCore | -----> (so on)
    +---------------+   (union)   +---------------+   (except)    +---------------+

    */
    fn build_select_core(&mut self, core: &SqlSelectCore) -> Result<Node, HaltReason> {
        let mut node: Node = Node::Nothing;

        let mut core_scope = Scope::new();

        // Source: The data flow starts from a source, which is a collection or a
        // subquery.
        if let Some(from) = &core.from {
            node = build_from(self, from, &mut core_scope)?;
        }

        // Filter: The source is then filtered, and the result is passed to the next
        // node.
        if let Some(predicate) = &core.r#where {
            let (expr, subqueries): (IntermediateExpr, Vec<Node>) = self.build_expr(
                predicate.as_ref(),
                InClause::Where,
                &mut core_scope,
                true,
                false,
            )?;
            node = Node::Filter {
                source: Box::new(node),
                predicate: expr,
                subqueries,
            }
        }

        // Pre-Aggregate: Once the filtering is done, it is time to explore all the
        // aggregates. This is done by collecting all the aggregates from the
        // expressions in the projection and the having clauses.
        let aggregates = collect_aggregates(core, self.interpreter)?;

        // Aggregate and Group By: In order to prepare an aggregate node, we need to
        // check if there are any grouping keys, too. We finally put the information
        // together and create the aggregate node.
        let group_by = if let Some(group_by) = &core.group_by {
            let mut keys = vec![];
            for key in group_by {
                let (expr, _) =
                    self.build_expr(key, InClause::GroupBy, &mut core_scope, false, false)?;
                keys.push(expr);
            }
            keys
        } else {
            vec![]
        };

        if !aggregates.is_empty() || !group_by.is_empty() {
            node = Node::Aggregate {
                source: Box::new(node),
                group_by,
                aggregates,
            };
        } else if let Some(having) = &core.having {
            // Fail fast if there is a HAVING clause without aggregation.
            return Err(HaltReason::Error(ExecutionError::Plan(
                PlannerError::HavingWithoutAggregationNotAllowed(having.get_span()),
            )));
        }

        // Projection: Projection is required to be done after the aggregate node, for the sake of
        // projecting aggregated data.
        if core.projection.as_slice() != [SqlProjection::All { collection: None }] {
            for projection in &core.projection {
                if let SqlProjection::Expr { expr, .. } = projection {
                    self.build_expr(expr, InClause::Projection, &mut core_scope, false, true)?;
                }
            }
            node = Node::Projection {
                source: Box::new(node),
                fields: core.projection.clone(),
            };
        }

        // PostProjection-Filter: After the aggregated data is projected, we can filter the
        // result using the HAVING clause. In earlier stages, we already collected
        // the aggregates from the projection and the having clause. As we already
        // have the aggregates, we can use them to filter the result.
        if let Some(having) = &core.having {
            let (expr, subqueries): (IntermediateExpr, Vec<Node>) =
                self.build_expr(having, InClause::Having, &mut core_scope, true, true)?;
            node = Node::Filter {
                source: Box::new(node),
                predicate: expr,
                subqueries,
            }
        }

        // We recursively build the compound queries (if any). The result of one
        // query is used as a source for another query.
        if let Some(compound) = &core.compound {
            node = Node::Compound {
                source: Box::new(node),
                operator: compound.operator.clone(),
                right: Box::new(self.build_select_core(&compound.core)?),
            }
        }
        Ok(node)
    }

    pub fn build_select(&mut self, query: &SqlSelect) -> Result<Node, HaltReason> {
        let mut node: Node = self.build_select_core(&query.core)?;
        let mut root_scope = Scope::new();

        if let Some(order_by) = &query.order_by {
            let mut order_key = vec![];

            for key in order_by {
                let (expr, _) =
                    self.build_expr(&key.expr, InClause::OrderBy, &mut root_scope, false, true)?;
                order_key.push((expr, key.ordering.clone()));
            }

            node = Node::Order {
                source: Box::new(node),
                key: order_key,
            };
        }

        if let Some(limit) = &query.limit {
            if let Some(offset) = &limit.offset {
                node = Node::Offset {
                    source: Box::new(node),
                    offset: self
                        .eval_constant(offset)?
                        .as_number()
                        .expect("Offset is not correct")
                        .floor() as usize,
                }
            }
            node = Node::Limit {
                source: Box::new(node),
                limit: self
                    .eval_constant(&limit.count)?
                    .as_number()
                    .expect("Limit is not correct")
                    .floor() as usize,
            }
        }

        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        engine::interpreter::{tests::create_test_interpreter},
        plan::{
            IntermediateExpr,
            planner::{InClause, Planner},
            scope::Scope,
        },
    };
    use lykiadb_lang::ast::{
        Identifier, IdentifierKind, Literal, Span,
        expr::{Expr, Operation},
        sql::{SqlDistinct, SqlProjection, SqlSelect, SqlSelectCore},
    };
    use std::sync::Arc;

    /// Helper function to create a test planner instance
    fn create_test_planner() -> Planner<'static> {
        let interpreter = Box::leak(Box::new(create_test_interpreter(None)));
        Planner::new(interpreter)
    }

    /// Helper function to create a test scope
    fn create_test_scope() -> Scope {
        Scope::new()
    }

    /// Helper function to create a simple identifier expression
    fn create_identifier_expr(name: &str) -> Expr {
        Expr::Variable {
            name: Identifier {
                name: name.to_string(),
                kind: IdentifierKind::Variable,
                span: Span::default(),
            },
            span: Span::default(),
            id: 0,
        }
    }

    /// Helper function to create a number literal expression
    fn create_number_expr(value: f64) -> Expr {
        Expr::Literal {
            value: Literal::Num(value),
            raw: value.to_string(),
            span: Span::default(),
            id: 0,
        }
    }

    /// Helper function to create a string literal expression
    fn create_string_expr(value: &str) -> Expr {
        Expr::Literal {
            value: Literal::Str(Arc::new(value.to_string())),
            raw: format!("\"{value}\""),
            span: Span::default(),
            id: 0,
        }
    }

    /// Helper function to create a function call expression
    fn create_call_expr(callee: &str, args: Vec<Expr>) -> Expr {
        Expr::Call {
            callee: Box::new(create_identifier_expr(callee)),
            args,
            span: Span::default(),
            id: 0,
        }
    }

    /// Helper function to create a field path expression
    fn create_field_path_expr(head: &str, tail: Vec<&str>) -> Expr {
        Expr::FieldPath {
            head: Identifier {
                name: head.to_string(),
                kind: IdentifierKind::Variable,
                span: Span::default(),
            },
            tail: tail
                .into_iter()
                .map(|t| Identifier {
                    name: t.to_string(),
                    kind: IdentifierKind::Variable,
                    span: Span::default(),
                })
                .collect(),
            span: Span::default(),
            id: 0,
        }
    }

    /// Helper function to create a subquery expression
    fn create_subquery_expr() -> Expr {
        Expr::Select {
            query: SqlSelect {
                core: SqlSelectCore {
                    distinct: SqlDistinct::ImplicitAll,
                    projection: vec![SqlProjection::All { collection: None }],
                    from: None,
                    r#where: None,
                    group_by: None,
                    having: None,
                    compound: None,
                },
                order_by: None,
                limit: None,
            },
            span: Span::default(),
            id: 0,
        }
    }

    // Helper macro to assert the result of build_expr
    macro_rules! assert_build_expr_result {
        ($result:expr, $expected_expr:expr, $expected_subquery_count:expr) => {
            assert!($result.is_ok());
            let (intermediate_expr, subqueries) = $result.unwrap();

            match intermediate_expr {
                IntermediateExpr::Expr { expr: boxed_expr } => {
                    assert_eq!(*boxed_expr, *$expected_expr);
                }
                _ => panic!("Expected IntermediateExpr::Expr"),
            }

            assert_eq!(subqueries.len(), $expected_subquery_count);
        };
    }

    #[test]
    fn test_build_expr_simple_literal() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();
        let expr = create_number_expr(42.0);

        let result = planner.build_expr(&expr, InClause::Where, &mut scope, false, false);

        // Use helper function to assert standard expectations
        assert_build_expr_result!(result, &expr, 0);
    }

    #[test]
    fn test_build_expr_string_literal() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();
        let expr = create_string_expr("hello");

        let result = planner.build_expr(&expr, InClause::Projection, &mut scope, false, true);

        assert_build_expr_result!(result, &expr, 0);
    }

    #[test]
    fn test_build_expr_identifier() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();
        let expr = create_identifier_expr("user_id");

        let result = planner.build_expr(&expr, InClause::Where, &mut scope, false, false);

        assert_build_expr_result!(result, &expr, 0);
    }

    #[test]
    fn test_build_expr_field_path() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();
        let expr = create_field_path_expr("user", vec!["name"]);

        let result = planner.build_expr(&expr, InClause::Projection, &mut scope, false, true);

        assert_build_expr_result!(result, &expr, 0);
    }

    #[test]
    fn test_build_expr_function_call() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();
        let expr = create_call_expr("upper", vec![create_string_expr("hello")]);

        let result = planner.build_expr(&expr, InClause::Projection, &mut scope, false, true);

        assert_build_expr_result!(result, &expr, 0);
    }

    #[test]
    fn test_build_expr_subquery_allowed() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();
        let expr = create_subquery_expr();

        let result = planner.build_expr(&expr, InClause::Where, &mut scope, true, false);

        assert_build_expr_result!(result, &expr, 1);
    }

    #[test]
    fn test_build_expr_subquery_not_allowed() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();
        let expr = create_subquery_expr();

        let result = planner.build_expr(&expr, InClause::Where, &mut scope, false, false);

        // Should return an error because subqueries are not allowed
        assert!(result.is_err());
    }

    #[test]
    fn test_build_expr_different_in_clause_values() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();
        let expr = create_identifier_expr("test_column");

        // Test all InClause variants
        let in_clauses = vec![
            InClause::Where,
            InClause::Projection,
            InClause::Having,
            InClause::GroupBy,
            InClause::OrderBy,
            InClause::JoinOn,
        ];

        for in_clause in in_clauses {
            let result = planner.build_expr(&expr, in_clause, &mut scope, false, true);

            assert_build_expr_result!(result, &expr, 0);
        }
    }

    #[test]
    fn test_build_expr_complex_expression() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();

        // Create a complex expression: upper(user.name) + " - " + user.id
        let expr = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(create_call_expr(
                    "upper",
                    vec![create_field_path_expr("user", vec!["name"])],
                )),
                operation: Operation::Add,
                right: Box::new(create_string_expr(" - ")),
                span: Span::default(),
                id: 0,
            }),
            operation: Operation::Add,
            right: Box::new(create_field_path_expr("user", vec!["id"])),
            span: Span::default(),
            id: 0,
        };

        let result = planner.build_expr(&expr, InClause::Projection, &mut scope, false, true);

        assert_build_expr_result!(result, &expr, 0);
    }

    #[test]
    fn test_build_expr_nested_subqueries() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();

        // Create an expression with multiple subqueries
        let subquery1 = create_subquery_expr();
        let subquery2 = create_subquery_expr();

        let expr = Expr::Binary {
            left: Box::new(subquery1),
            operation: Operation::And,
            right: Box::new(subquery2),
            span: Span::default(),
            id: 0,
        };

        let result = planner.build_expr(&expr, InClause::Where, &mut scope, true, false);

        assert_build_expr_result!(result, &expr, 2);
    }

    #[test]
    fn test_build_expr_scope_validation() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();

        // Test that the scope is passed correctly to SqlExprReducer
        let expr = create_field_path_expr("nonexistent_table", vec!["column"]);

        let result = planner.build_expr(&expr, InClause::Where, &mut scope, false, false);

        // Should still succeed as the method returns the expression regardless of scope validation
        assert_build_expr_result!(result, &expr, 0);
    }

    #[test]
    fn test_build_expr_aggregate_function_allowed() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();

        // Create an aggregate function call like AVG(*)
        let expr = create_call_expr("avg", vec![create_identifier_expr("*")]);

        let result = planner.build_expr(&expr, InClause::Projection, &mut scope, false, true);

        assert_build_expr_result!(result, &expr, 0);
    }

    #[test]
    fn test_build_expr_boolean_literal() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();

        let expr = Expr::Literal {
            raw: "true".to_string(),
            value: Literal::Bool(true),
            id: 0,
            span: Span::default(),
        };

        let result = planner.build_expr(&expr, InClause::Where, &mut scope, false, false);

        assert_build_expr_result!(result, &expr, 0);
    }

    #[test]
    fn test_build_expr_nested_function_calls() {
        let mut planner = create_test_planner();
        let mut scope = create_test_scope();

        // Create nested function calls: upper(trim(user.name))
        let expr = create_call_expr(
            "upper",
            vec![create_call_expr(
                "trim",
                vec![create_field_path_expr("user", vec!["name"])],
            )],
        );

        let result = planner.build_expr(&expr, InClause::Projection, &mut scope, false, true);

        assert_build_expr_result!(result, &expr, 0);
    }
}
