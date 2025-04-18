use std::collections::HashSet;

use crate::{
    engine::{
        error::ExecutionError,
        interpreter::{Aggregation, HaltReason, Interpreter},
    },
    value::{callable::CallableKind, RV},
};

use lykiadb_lang::ast::{
    expr::Expr,
    sql::{SqlFrom, SqlJoinType, SqlProjection, SqlSelect, SqlSelectCore, SqlSource},
    visitor::VisitorMut,
    Spanned,
};

use super::{scope::Scope, IntermediateExpr, Node, Plan, PlannerError};

pub struct Planner<'a> {
    interpreter: &'a mut Interpreter,
}


impl<'a> Planner<'a> {
    fn collect_aggregates_from_expr(
        &mut self,
        expr: &Expr
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
            },
            //
            Expr::Call { callee, args, .. } => {
                let mut result: Vec<Aggregation> = vec![];
                
                let callee_val = self.interpreter.eval(callee);

                if let Ok(RV::Callable(callable)) = &callee_val {
                    if let CallableKind::Aggregator(agg_name) = &callable.kind  {
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
                    .flat_map(|x| x.unwrap()).collect();

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
            },
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
            _ => panic!("Not implemented yet."),
        }
    }

    fn collect_aggregates(&mut self, core: &SqlSelectCore) ->  Result<Vec<Aggregation>, HaltReason> {
        let mut aggregates: HashSet<Aggregation> = HashSet::new();

        for projection in &core.projection {
            if let SqlProjection::Expr { expr, .. } = projection {
                let found = self.collect_aggregates_from_expr(expr)?;
                for agg in found {
                    aggregates.insert(agg);
                }
            }
        }

        let no_dup = aggregates.drain().collect();

        Ok(no_dup)
    }

    fn build_select_core(&mut self, core: &SqlSelectCore) -> Result<Node, HaltReason> {
        let mut node: Node = Node::Nothing;

        let mut parent_scope = Scope::new();

        // FROM/JOIN
        if let Some(from) = &core.from {
            node = self.build_from(from, &mut parent_scope)?;
        }

        // WHERE
        if let Some(predicate) = &core.r#where {
            let (expr, subqueries): (IntermediateExpr, Vec<Node>) =
                self.build_expr(predicate.as_ref(), true, false)?;
            node = Node::Filter {
                source: Box::new(node),
                predicate: expr,
                subqueries,
            }
        }

        // AGGREGATES
        let aggregates = self.collect_aggregates(&core)?;

        // GROUP BY
        if let Some(group_by) = &core.group_by {
            let mut keys = vec![];
            for key in group_by {
                let (expr, _) = self.build_expr(key, false, true)?;
                keys.push(expr);
            }
            node = Node::Aggregate { 
                source: Box::new(node),
                group_by: keys,
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

        // HAVING

        // COMPOUND
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

    fn build_from(&mut self, from: &SqlFrom, parent_scope: &mut Scope) -> Result<Node, HaltReason> {
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
                let mut node = self.build_from(froms.next().unwrap(), &mut scope)?;
                for right in froms {
                    node = Node::Join {
                        left: Box::new(node),
                        join_type: SqlJoinType::Cross,
                        right: Box::new(self.build_from(right, &mut scope)?),
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
                    left: Box::new(self.build_from(left, &mut scope)?),
                    join_type: join_type.clone(),
                    right: Box::new(self.build_from(right, &mut scope)?),
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
