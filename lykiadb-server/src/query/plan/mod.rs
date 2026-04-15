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

use crate::value::{RV, array::RVArray, object::RVObject, callable::AggregatorFactory};
use derivative::Derivative;
use std::sync::Arc;

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
    pub fn to_object(&self) -> RV<'v> {
        match self {
            Plan::Select(node) => node.to_object(),
        }
    }
}

impl<'v> Display for Plan<'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self.to_object()) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

macro_rules! rv_object {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut map: rustc_hash::FxHashMap<String, RV<'v>> = rustc_hash::FxHashMap::default();
        $( map.insert(String::from($key), $val); )*
        RV::Object(RVObject::from_map(map))
    }};
}

macro_rules! rv_str {
    ($s:expr) => {
        RV::Str(Arc::new($s.to_string()))
    };
}

impl<'v> Node<'v> {
    fn to_object(&self) -> RV<'v> {
        match self {
            Node::Nothing => rv_object! { "type" => rv_str!("nothing") },

            Node::Scan { source, .. } => rv_object!{
                "type" => rv_str!("scan"),
                "collection" => rv_str!(source.name.name),
                "alias" => rv_str!(source.alias.as_ref().map(|a| a.name.clone()).unwrap_or_else(|| source.name.name.clone())),
            },

            Node::EvalScan { source, .. } => rv_object!{
                "type" => rv_str!("eval_scan"),
                "expr" => rv_str!(source.expr.to_string()),
                "alias" => rv_str!(source.alias.name),
            },

            Node::Filter { source, predicate, subqueries } => {
                let mut obj = rv_object!{
                    "type" => rv_str!("filter"),
                    "predicate" => rv_str!(predicate.to_string()),
                    "source" => source.to_object(),
                };
                /*if !subqueries.is_empty() {
                    obj["subqueries"] = rv_object!{ 
                        "type" => rv_str!("subqueries"), 
                        "queries" => subqueries.iter().map(|s| s.to_object()).collect() 
                    };
                }*/
                obj
            }

            Node::Projection { source, fields } => {
                let field_strs = RVArray::from_vec(fields.iter().map(|f| match f {
                    SqlProjection::All { collection: None } => rv_str!("*".to_string()),
                    SqlProjection::All { collection: Some(c) } => rv_str!(format!("{}.*", c.name)),
                    SqlProjection::Expr { expr, alias: None } => rv_str!(expr.to_string()),
                    SqlProjection::Expr { expr, alias: Some(a) } => rv_str!(format!("{} as {}", expr, a.name)),
                }).collect());
                rv_object!{
                    "@type" => rv_str!("projection"),
                    "fields" => RV::Array(field_strs),
                    "source" => source.to_object(),
                }
            }

            Node::Aggregate { source, group_by, aggregates } => rv_object!{
                "@type" => rv_str!("aggregate"),
                "group_by" => RV::Array(
                        RVArray::from_vec(
                            group_by.iter().map(|e| rv_str!(e.to_string())).collect::<Vec<_>>(),
                        )
                    ),
                "aggregates" => RV::Array(
                    RVArray::from_vec(
                        aggregates.iter().map(|a| rv_str!(a.to_string())).collect::<Vec<_>>(),
                    )
                ),
                "source" => source.to_object(),
            },

            Node::Order { source, key } => {
                let key_json= RV::Array(
                        RVArray::from_vec(
                                    key.iter().map(|(expr, ord)| {
                            let ord_str = match ord {
                                SqlOrdering::Asc => "asc",
                                SqlOrdering::Desc => "desc",
                            };
                            RV::Array(
                            RVArray::from_vec(vec![rv_str!(expr.to_string()), rv_str!(ord_str)]))
                        }).collect()
                    )
                );
                rv_object! {
                    "@type" => rv_str!("order"),
                    "key" => key_json,
                    "source" => source.to_object(),
                }
            }

            Node::Limit { source, limit } => rv_object!{
                "@type" => rv_str!("limit"),
                "count" => RV::Int64(*limit as i64),
                "source" => source.to_object(),
            },

            Node::Offset { source, offset } => rv_object!{
                "@type" => rv_str!("offset"),
                "count" => RV::Int64(*offset as i64),
                "source" => source.to_object(),
            },

            Node::Join { left, join_type, right, constraint } => {
                let join_type_str = match join_type {
                    SqlJoinType::Inner => "inner",
                    SqlJoinType::Cross => "cross",
                    SqlJoinType::Left => "left",
                    SqlJoinType::Right => "right",
                };
                rv_object!{
                    "@type" => rv_str!("join"),
                    "join_type" => rv_str!(join_type_str),
                    "constraint" => constraint.as_ref().map(|c| rv_str!(c.to_string())).unwrap_or(RV::Undefined),
                    "left" => left.to_object(),
                    "right" => right.to_object(),
                }
            }

            Node::Compound { source, operator, right } => {
                let op_str = match operator {
                    SqlCompoundOperator::Union => "union",
                    SqlCompoundOperator::UnionAll => "union_all",
                    SqlCompoundOperator::Intersect => "intersect",
                    SqlCompoundOperator::Except => "except",
                };
                rv_object!{
                    "@type" => rv_str!("compound"),
                    "operator" => rv_str!(op_str),
                    "source" => source.to_object(),
                    "right" => right.to_object(),
                }
            }

            Node::Subquery { source, alias } => rv_object! {
                "@type" => rv_str!("subquery"),
                "alias" => rv_str!(alias.name),
                "source" => source.to_object(),
            },
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
