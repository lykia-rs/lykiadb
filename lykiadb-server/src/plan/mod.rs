use std::fmt::Display;

use lykiadb_lang::ast::{
    Identifier, Span,
    expr::Expr,
    sql::{
        SqlCollectionIdentifier, SqlCompoundOperator, SqlExpressionSource, SqlJoinType,
        SqlOrdering, SqlProjection,
    },
};
use serde::{Deserialize, Serialize};

use crate::{engine::interpreter::Aggregation, value::RV};

mod aggregation;
mod from;
pub mod planner;
mod scope;
mod expr;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum PlannerError {
    NestedAggregationNotAllowed(Span),
    SubqueryNotAllowed(Span),
    ObjectNotFoundInScope(Identifier),
    DuplicateObjectInScope(Identifier),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IntermediateExpr {
    Constant(RV),
    Expr { expr: Expr },
}

impl Display for IntermediateExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntermediateExpr::Constant(rv) => write!(f, "{:?}", rv),
            IntermediateExpr::Expr { expr, .. } => {
                write!(f, "{}", expr)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Plan {
    Select(Node),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Compound {
        source: Box<Node>,
        operator: SqlCompoundOperator,
        right: Box<Node>,
    },

    Aggregate {
        source: Box<Node>,
        group_by: Vec<IntermediateExpr>,
        aggregates: Vec<Aggregation>,
    },

    Filter {
        source: Box<Node>,
        predicate: IntermediateExpr,
        subqueries: Vec<Node>,
    },

    Projection {
        source: Box<Node>,
        fields: Vec<SqlProjection>,
    },

    Limit {
        source: Box<Node>,
        limit: usize,
    },

    Offset {
        source: Box<Node>,
        offset: usize,
    },

    Order {
        source: Box<Node>,
        key: Vec<(IntermediateExpr, SqlOrdering)>,
    },

    Values {
        rows: Vec<Vec<IntermediateExpr>>,
    },

    Scan {
        source: SqlCollectionIdentifier,
        filter: Option<IntermediateExpr>,
    },

    EvalScan {
        source: SqlExpressionSource,
        filter: Option<IntermediateExpr>,
    },

    Join {
        left: Box<Node>,
        join_type: SqlJoinType,
        right: Box<Node>,
        constraint: Option<IntermediateExpr>,
    },

    Subquery {
        source: Box<Node>,
        alias: Option<Identifier>,
    },

    Nothing,
}

impl Display for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Plan::Select(node) => write!(f, "{}", node),
        }
    }
}

impl Node {
    const TAB: &'static str = "  ";
    const NEWLINE: &'static str = "\n";

    fn _fmt_recursive(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        let indent_str = Self::TAB.repeat(indent);
        match self {
            Node::Nothing => write!(f, "{}- nothing{}", indent_str, Self::NEWLINE),
            Node::Order { source, key } => {
                let key_description = key
                    .iter()
                    .map(|(expr, ordering)| format!("({}, {:?})", expr, ordering))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(
                    f,
                    "{}- order [{}]{}",
                    indent_str,
                    key_description,
                    Self::NEWLINE
                )?;
                source._fmt_recursive(f, indent + 1)
            }
            Node::Projection { source, fields } => {
                let fields_description = fields
                    .iter()
                    .map(|field| match field {
                        SqlProjection::All { collection } => {
                            if let Some(c) = collection.as_ref() {
                                return format!("* in {}", c.name);
                            }
                            "*".to_string()
                        }
                        SqlProjection::Expr { expr, alias } => {
                            if let Some(alias) = alias {
                                return format!("{} as {}", expr, alias.name);
                            }
                            format!("{} as {}", expr, expr)
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(
                    f,
                    "{}- project [{}]{}",
                    indent_str,
                    fields_description,
                    Self::NEWLINE
                )?;

                source._fmt_recursive(f, indent + 1)
            }
            Node::Filter {
                source,
                predicate,
                subqueries,
            } => {
                write!(f, "{}- filter [{}]{}", indent_str, predicate, Self::NEWLINE)?;
                if !subqueries.is_empty() {
                    write!(f, "{}  > subqueries{}", indent_str, Self::NEWLINE)?;
                    subqueries
                        .iter()
                        .try_for_each(|subquery| subquery._fmt_recursive(f, indent + 2))?;
                }
                source._fmt_recursive(f, indent + 1)
            }
            Node::Subquery { source, alias } => {
                write!(
                    f,
                    "{}- subquery [{}]{}",
                    indent_str,
                    alias
                        .as_ref()
                        .map(|x| x.name.clone())
                        .unwrap_or("unnamed".to_string()),
                    Self::NEWLINE
                )?;
                source._fmt_recursive(f, indent + 1)
            }
            Node::Scan { source, filter } => {
                write!(
                    f,
                    "{}- scan [{} as {}]{}",
                    indent_str,
                    source.name,
                    source.alias.as_ref().unwrap_or(&source.name),
                    Self::NEWLINE
                )
            }
            Node::Compound {
                source,
                operator,
                right,
            } => {
                write!(
                    f,
                    "{}- compound [type={:?}]{}",
                    indent_str,
                    operator,
                    Self::NEWLINE
                )?;
                source._fmt_recursive(f, indent + 1)?;
                right._fmt_recursive(f, indent + 1)
            }
            Node::Limit { source, limit } => {
                write!(
                    f,
                    "{}- limit [count={}]{}",
                    indent_str,
                    limit,
                    Self::NEWLINE
                )?;
                source._fmt_recursive(f, indent + 1)
            }
            Node::Offset { source, offset } => {
                write!(
                    f,
                    "{}- offset [count={}]{}",
                    indent_str,
                    offset,
                    Self::NEWLINE
                )?;
                source._fmt_recursive(f, indent + 1)
            }
            Node::Join {
                left,
                join_type,
                right,
                constraint,
            } => {
                write!(
                    f,
                    "{}- join [type={:?}, {}]{}",
                    indent_str,
                    join_type,
                    constraint
                        .as_ref()
                        .map(|x| x.to_string())
                        .unwrap_or("None".to_string()),
                    Self::NEWLINE
                )?;
                left._fmt_recursive(f, indent + 1)?;
                right._fmt_recursive(f, indent + 1)
            }
            Node::EvalScan { source, filter } => {
                write!(
                    f,
                    "{}- eval_scan [{}]{}",
                    indent_str,
                    source.expr,
                    Self::NEWLINE
                )
            }
            Node::Aggregate {
                source,
                group_by,
                aggregates,
            } => {
                let group_by_description = group_by
                    .iter()
                    .map(|expr| expr.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                let aggregates_description = aggregates
                    .iter()
                    .map(|aggregate| {
                        let args = aggregate
                            .args
                            .iter()
                            .map(|arg| arg.to_string())
                            .collect::<Vec<String>>()
                            .join(", ");
                        format!("{}({})", aggregate.name, args)
                    })
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(
                    f,
                    "{}- aggregate [group_by=[{}], aggregates=[{}]]{}",
                    indent_str,
                    group_by_description,
                    aggregates_description,
                    Self::NEWLINE
                )?;
                source._fmt_recursive(f, indent + 1)
            }
            _ => "<NotImplementedYet>".fmt(f),
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._fmt_recursive(f, 0)
    }
}
