use crate::{
    engine::interpreter::{HaltReason, Interpreter},
    value::RV,
};

use lykiadb_lang::ast::{
    expr::Expr, sql::{SqlProjection, SqlSelect, SqlSelectCore}, visitor::{ExprVisitor, VisitorMut}, Spanned
};

use super::{
    aggregation::collect_aggregates, expr::SqlExprReducer, from::build_from, scope::Scope, IntermediateExpr, Node, Plan, PlannerError
};

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

        println!("CurrentScope\n{:?}", core_scope);

        // Filter: The source is then filtered, and the result is passed to the next
        // node.
        if let Some(predicate) = &core.r#where {
            let (expr, subqueries): (IntermediateExpr, Vec<Node>) =
                self.build_expr(predicate.as_ref(), &mut core_scope, true, false)?;
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
                let (expr, _) = self.build_expr(key, &mut core_scope, false, true)?;
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
        }

        // Projection: Of course, projection is an essential part of the data flow,
        // and it is required to be done after the aggregate node, for the sake of
        // projecting aggregated data.
        if core.projection.as_slice() != [SqlProjection::All { collection: None }] {
            for projection in &core.projection {
                if let SqlProjection::Expr { expr, .. } = projection {
                    self.build_expr(expr,
                         &mut core_scope, 
                         false, true)?;
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
        if core.having.is_some() {
            let (expr, subqueries): (IntermediateExpr, Vec<Node>) =
                self.build_expr(core.having.as_ref().unwrap(), &mut core_scope, true, false)?;
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

    fn eval_constant(&mut self, expr: &Expr) -> Result<RV, HaltReason> {
        self.interpreter.visit_expr(expr)
    }

    pub fn build_expr(
        &mut self,
        expr: &Expr,
        scope: &mut Scope,
        allow_subqueries: bool,
        allow_aggregates: bool,
    ) -> Result<(IntermediateExpr, Vec<Node>), HaltReason> {        
        let mut reducer: SqlExprReducer = SqlExprReducer::new(
            // self,
            allow_subqueries,
        );

        let mut visitor = ExprVisitor::<SqlSelect, HaltReason>::new(
            &mut reducer,
        );

        let selects = visitor.visit(expr)?;

        let mut subqueries = vec![];

        for subquery in &selects {
            subqueries.push(self.build_select(subquery)?);
        }

        Ok((IntermediateExpr::Expr { expr: expr.clone() }, subqueries))
    }

    pub fn build_select(&mut self, query: &SqlSelect) -> Result<Node, HaltReason> {
        let mut node: Node = self.build_select_core(&query.core)?;
        let mut root_scope = Scope::new();
        
        if let Some(order_by) = &query.order_by {
            let mut order_key = vec![];

            for key in order_by {
                let (expr, _) = self.build_expr(&key.expr, &mut root_scope, false, true)?;
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
