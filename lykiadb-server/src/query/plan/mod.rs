use std::fmt::Display;

use lykiadb_lang::ast::{
    Identifier,
    expr::Expr,
    sql::{
        SqlCollectionIdentifier, SqlCompoundOperator, SqlExpressionSource, SqlJoinType,
        SqlOrdering, SqlProjection,
    },
};
use serde::{Deserialize, Serialize};

use crate::value::{RV, callable::AggregatorFactory};
use derivative::Derivative;

mod aggregation;
pub mod error;
mod expr;
mod from;
pub mod planner;
mod scope;

crate::register_tests!("lykiadb-server/src/query/plan/tests");

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IntermediateExpr<'v> {
    Constant(RV<'v>),
    Expr { expr: Box<Expr> },
}

impl<'v> Display for IntermediateExpr<'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntermediateExpr::Constant(rv) => write!(f, "{rv:?}"),
            IntermediateExpr::Expr { expr, .. } => {
                write!(f, "{expr}")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Plan<'v> {
    Select(Node<'v>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node<'v> {
    Compound {
        source: Box<Node<'v>>,
        operator: SqlCompoundOperator,
        right: Box<Node<'v>>,
    },

    Aggregate {
        source: Box<Node<'v>>,
        group_by: Vec<IntermediateExpr<'v>>,
        aggregates: Vec<Aggregation<'v>>,
    },

    Filter {
        source: Box<Node<'v>>,
        predicate: IntermediateExpr<'v>,
        subqueries: Vec<Node<'v>>,
    },

    Projection {
        source: Box<Node<'v>>,
        fields: Vec<SqlProjection>,
    },

    Limit {
        source: Box<Node<'v>>,
        limit: usize,
    },

    Offset {
        source: Box<Node<'v>>,
        offset: usize,
    },

    Order {
        source: Box<Node<'v>>,
        key: Vec<(IntermediateExpr<'v>, SqlOrdering)>,
    },

    Scan {
        source: SqlCollectionIdentifier,
        filter: Option<IntermediateExpr<'v>>,
    },

    EvalScan {
        source: SqlExpressionSource,
        filter: Option<IntermediateExpr<'v>>,
    },

    Join {
        left: Box<Node<'v>>,
        join_type: SqlJoinType,
        right: Box<Node<'v>>,
        constraint: Option<IntermediateExpr<'v>>,
    },

    Subquery {
        source: Box<Node<'v>>,
        alias: Identifier,
    },

    Nothing,
}

impl<'v> Plan<'v> {
    fn to_plan_json(&self) -> serde_json::Value {
        match self {
            Plan::Select(node) => node.to_plan_json(),
        }
    }
}

impl<'v> Display for Plan<'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self.to_plan_json()) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

impl<'v> Node<'v> {
    fn to_plan_json(&self) -> serde_json::Value {
        use serde_json::json;
        match self {
            Node::Nothing => json!({ "type": "nothing" }),

            Node::Scan { source, .. } => json!({
                "type": "scan",
                "collection": source.name.name,
                "alias": source.alias.as_ref().map(|a| a.name.clone()).unwrap_or_else(|| source.name.name.clone()),
            }),

            Node::EvalScan { source, .. } => json!({
                "type": "eval_scan",
                "expr": source.expr.to_string(),
                "alias": source.alias.name,
            }),

            Node::Filter { source, predicate, subqueries } => {
                let mut obj = json!({
                    "type": "filter",
                    "predicate": predicate.to_string(),
                    "source": source.to_plan_json(),
                });
                if !subqueries.is_empty() {
                    obj["subqueries"] = subqueries.iter().map(|s| s.to_plan_json()).collect();
                }
                obj
            }

            Node::Projection { source, fields } => {
                let field_strs: Vec<String> = fields.iter().map(|f| match f {
                    SqlProjection::All { collection: None } => "*".to_string(),
                    SqlProjection::All { collection: Some(c) } => format!("{}.*", c.name),
                    SqlProjection::Expr { expr, alias: None } => expr.to_string(),
                    SqlProjection::Expr { expr, alias: Some(a) } => format!("{} as {}", expr, a.name),
                }).collect();
                json!({
                    "type": "projection",
                    "fields": field_strs,
                    "source": source.to_plan_json(),
                })
            }

            Node::Aggregate { source, group_by, aggregates } => json!({
                "type": "aggregate",
                "group_by": group_by.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
                "aggregates": aggregates.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
                "source": source.to_plan_json(),
            }),

            Node::Order { source, key } => {
                let key_json: Vec<serde_json::Value> = key.iter().map(|(expr, ord)| {
                    let ord_str = match ord {
                        SqlOrdering::Asc => "asc",
                        SqlOrdering::Desc => "desc",
                    };
                    json!([expr.to_string(), ord_str])
                }).collect();
                json!({
                    "type": "order",
                    "key": key_json,
                    "source": source.to_plan_json(),
                })
            }

            Node::Limit { source, limit } => json!({
                "type": "limit",
                "count": limit,
                "source": source.to_plan_json(),
            }),

            Node::Offset { source, offset } => json!({
                "type": "offset",
                "count": offset,
                "source": source.to_plan_json(),
            }),

            Node::Join { left, join_type, right, constraint } => {
                let join_type_str = match join_type {
                    SqlJoinType::Inner => "inner",
                    SqlJoinType::Cross => "cross",
                    SqlJoinType::Left => "left",
                    SqlJoinType::Right => "right",
                };
                json!({
                    "type": "join",
                    "join_type": join_type_str,
                    "constraint": constraint.as_ref().map(|c| c.to_string()),
                    "left": left.to_plan_json(),
                    "right": right.to_plan_json(),
                })
            }

            Node::Compound { source, operator, right } => {
                let op_str = match operator {
                    SqlCompoundOperator::Union => "union",
                    SqlCompoundOperator::UnionAll => "union_all",
                    SqlCompoundOperator::Intersect => "intersect",
                    SqlCompoundOperator::Except => "except",
                };
                json!({
                    "type": "compound",
                    "operator": op_str,
                    "source": source.to_plan_json(),
                    "right": right.to_plan_json(),
                })
            }

            Node::Subquery { source, alias } => json!({
                "type": "subquery",
                "alias": alias.name,
                "source": source.to_plan_json(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Aggregation<'v> {
    pub name: String,
    #[serde(skip)]
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub callable: Option<AggregatorFactory<'v>>,
    pub args: Vec<Expr>,
    pub call_expr: Expr,
    pub call_sign: String,
}

impl<'v> Aggregation<'v> {
    pub fn new(
        agg_name: &str,
        agg_factory: &AggregatorFactory<'v>,
        args: &Vec<Expr>,
        expr: &Expr,
    ) -> Aggregation<'v> {
        Aggregation {
            name: agg_name.to_string(),
            callable: Some(*agg_factory),
            args: args.clone(),
            call_expr: expr.clone(),
            call_sign: expr.sign(),
        }
    }
}

impl<'v> Display for Aggregation<'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.args
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
