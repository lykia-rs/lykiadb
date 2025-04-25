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
    sql::{SqlFrom, SqlJoinType, SqlProjection, SqlSelect, SqlSelectCore, SqlSource},
    visitor::VisitorMut,
};

use super::{IntermediateExpr, Node, Plan, PlannerError, scope::Scope};

pub struct Planner<'a> {
    interpreter: &'a mut Interpreter,
}

impl<'a> Planner<'a> {
    fn collect_aggregates_from_expr(
        &mut self,
        expr: &Expr,
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
                let rleft = self.collect_aggregates_from_expr(left);
                let rright = self.collect_aggregates_from_expr(right);

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
            Expr::Grouping { expr, .. }
            | Expr::Unary { expr, .. }
            | Expr::Assignment { expr, .. } => {
                let r = self.collect_aggregates_from_expr(expr);
                if let Ok(v) = r {
                    return Ok(v);
                }
                Ok(vec![])
            }
            //
            Expr::Call { callee, args, .. } => {
                let mut result: Vec<Aggregation> = vec![];

                let callee_val = self.interpreter.eval(callee);

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
                    .map(|x| self.collect_aggregates_from_expr(x))
                    .flat_map(|x| x.unwrap())
                    .collect();

                if rargs.len() > 0 {
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
                let rlower = self.collect_aggregates_from_expr(lower);
                let rupper = self.collect_aggregates_from_expr(upper);
                let rsubject = self.collect_aggregates_from_expr(subject);

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
                let robject = self.collect_aggregates_from_expr(object);
                if let Ok(v) = robject {
                    return Ok(v);
                }
                Ok(vec![])
            }
        }
    }
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

    The end result is a computation graph, that can be easily combined with other computation graphs.
    A typical example is a compound query, where the result of one query is used as a source for another query.
    The data flow is as follows:

    +---------------+             +---------------+               +---------------+
    | SqlSelectCore | ----------> | SqlSelectCore | ------------> | SqlSelectCore | -----> (so on)
    +---------------+   (union)   +---------------+   (except)    +---------------+

    */

    // Source: The data flow starts from a source, which is a collection or a subquery.

    // Filter: The source is then filtered, and the result is passed to the next node.

    // Pre-Aggregate: Once the filtering is done, it is time to explore all the aggregates. This is done by collecting all the aggregates from the expressions in the projection and the having clauses.

    // Aggregate and Group By: In order to prepare an aggregate node, we need to check if there are any grouping keys, too. We finally put the information together and create the aggregate node.

    // Projection: Of course, projection is an essential part of the data flow, and it is required to be done after the aggregate node, for the sake of projecting aggregated data.

    // Post-Filter: After the aggregated data is projected, we can filter the result using the HAVING clause. In earlier stages, we already collected the aggregates from the projection and the having clause. As we already have the aggregates, we can use them to filter the result.

    fn build_select_core(&mut self, core: &SqlSelectCore) -> Result<Node, HaltReason> {
        let mut node: Node = Node::Nothing;

        let mut parent_scope = Scope::new();

        // Data flow starts from a source, which is a collection or a subquery.
        if let Some(from) = &core.from {
            node = self.build_source(from, &mut parent_scope)?;
        }

        // The source is then filtered, and the result is passed to the next node.
        if let Some(predicate) = &core.r#where {
            let (expr, subqueries): (IntermediateExpr, Vec<Node>) =
                self.build_expr(predicate.as_ref(), true, false)?;
            node = Node::Filter {
                source: Box::new(node),
                predicate: expr,
                subqueries,
            }
        }

        // Once the filtering is done, it is time to explore all the aggregates.
        // This is done by collecting all the aggregates from the expressions in the projection and the having clauses.
        let aggregates = self.collect_aggregates(&core)?;

        // In order to prepare an aggregate node, we need to check if there are any grouping keys, too.
        let group_by = if let Some(group_by) = &core.group_by {
            let mut keys = vec![];
            for key in group_by {
                let (expr, _) = self.build_expr(key, false, true)?;
                keys.push(expr);
            }
            keys
        } else {
            vec![]
        };

        if aggregates.len() > 0 || group_by.len() > 0 {
            node = Node::Aggregate {
                source: Box::new(node),
                group_by,
                aggregates,
            };
        }

        // PROJECTION
        if core.projection.as_slice() != [SqlProjection::All { collection: None }] {
            for projection in &core.projection {
                if let SqlProjection::Expr { expr, .. } = projection {
                    self.build_expr(expr, false, true)?;
                }
            }
            node = Node::Projection {
                source: Box::new(node),
                fields: core.projection.clone(),
            };
        }

        // After the aggregated data is projected, we can filter the result using the HAVING clause.
        // In earlier stages, we already collected the aggregates from the projection and the having clause.
        // As we already have the aggregates, we can use them to filter the result.
        if core.having.is_some() {
            let (expr, subqueries): (IntermediateExpr, Vec<Node>) =
                self.build_expr(core.having.as_ref().unwrap(), true, false)?;
            node = Node::Filter {
                source: Box::new(node),
                predicate: expr,
                subqueries,
            }
        }

        // We recursively build the compound queries (if any).
        // The result of one query is used as a source for another query.
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

    fn build_expr(
        &mut self,
        expr: &Expr,
        allow_subqueries: bool,
        allow_aggregates: bool,
    ) -> Result<(IntermediateExpr, Vec<Node>), HaltReason> {
        // TODO(vck): Implement this

        let mut subqueries: Vec<Node> = vec![];

        let result = expr.walk::<(), HaltReason>(&mut |e: &Expr| match e {
            Expr::Get { object, name, .. } => {
                // println!("Get {}.({})", object, name);
                None
            }
            Expr::FieldPath { head, tail, .. } => {
                /* println!(
                    "FieldPath {} {}",
                    head,
                    tail.iter().map(|x| x.to_string() + " ").collect::<String>()
                ); */
                None
            }
            Expr::Call { callee, args, .. } => {
                // println!("Call {}({:?})", callee, args);
                None
            }
            Expr::Select { query, .. } => {
                if !allow_subqueries {
                    return Some(Err(HaltReason::Error(ExecutionError::Plan(
                        PlannerError::SubqueryNotAllowed(expr.get_span()),
                    ))));
                }
                let subquery = self.build_select(query);
                subqueries.push(subquery.unwrap());
                None
            }
            _ => Some(Ok(())),
        });

        if let Some(Err(err)) = result {
            return Err(err);
        }

        Ok((IntermediateExpr::Expr { expr: expr.clone() }, subqueries))
    }

    fn build_select(&mut self, query: &SqlSelect) -> Result<Node, HaltReason> {
        let mut node: Node = self.build_select_core(&query.core)?;

        if let Some(order_by) = &query.order_by {
            let mut order_key = vec![];

            for key in order_by {
                let (expr, _) = self.build_expr(&key.expr, false, true)?;
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

    // Collects all the aggregates from the projection and the having clause.
    // The aggregates are stored in a HashSet to avoid duplicates and then returned as a Vec<Aggregation>.
    // For the time being, we only find aggregates in the projection and the having clause.
    fn collect_aggregates(&mut self, core: &SqlSelectCore) -> Result<Vec<Aggregation>, HaltReason> {
        let mut aggregates: HashSet<Aggregation> = HashSet::new();

        for projection in &core.projection {
            if let SqlProjection::Expr { expr, .. } = projection {
                let found = self.collect_aggregates_from_expr(expr)?;
                for agg in found {
                    aggregates.insert(agg);
                }
            }
        }

        if let Some(expr) = &core.having {
            let found = self.collect_aggregates_from_expr(expr)?;
            for agg in found {
                aggregates.insert(agg);
            }
        }

        let no_dup = aggregates.drain().collect();

        Ok(no_dup)
    }

    // The source can be of following types:

    // - Collection: A collection of data, like a table.
    // - Expr: An expression that returns a set of data.
    // - Subquery: A subquery that returns a set of data.
    // - Join: A join between two or more sources.
    // - Group: Cartesian product of two or more sources.
    fn build_source(
        &mut self,
        from: &SqlFrom,
        parent_scope: &mut Scope,
    ) -> Result<Node, HaltReason> {
        let mut scope = Scope::new();

        let node = match from {
            SqlFrom::Source(source) => {
                let wrapped = match source {
                    SqlSource::Collection(ident) => Node::Scan {
                        source: ident.clone(),
                        filter: None,
                    },
                    SqlSource::Expr(expr) => Node::EvalScan {
                        source: expr.clone(),
                        filter: None,
                    },
                };

                if let Err(err) = scope.add_source(source.clone()) {
                    return Err(HaltReason::Error(ExecutionError::Plan(err)));
                }

                Ok(wrapped)
            }
            SqlFrom::Select { subquery, alias } => {
                let node = Node::Subquery {
                    source: Box::new(self.build_select(subquery)?),
                    alias: alias.clone(),
                };
                Ok(node)
            }
            SqlFrom::Group { values } => {
                let mut froms = values.iter();
                let mut node = self.build_source(froms.next().unwrap(), &mut scope)?;
                for right in froms {
                    node = Node::Join {
                        left: Box::new(node),
                        join_type: SqlJoinType::Cross,
                        right: Box::new(self.build_source(right, &mut scope)?),
                        constraint: None,
                    }
                }
                Ok(node)
            }
            SqlFrom::Join {
                left,
                join_type,
                right,
                constraint,
            } => {
                let constraint = constraint
                    .as_ref()
                    .map(|x| self.build_expr(x, false, false))
                    .transpose()?;

                Ok(Node::Join {
                    left: Box::new(self.build_source(left, &mut scope)?),
                    join_type: join_type.clone(),
                    right: Box::new(self.build_source(right, &mut scope)?),
                    constraint: constraint.map(|x| x.0),
                })
            }
        };

        if let Err(err) = parent_scope.merge(scope) {
            return Err(HaltReason::Error(ExecutionError::Plan(err)));
        }

        node
    }
}
