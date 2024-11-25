use std::fmt::Display;

use lykiadb_lang::{
    ast::{
        expr::Expr,
        sql::{SqlCollectionIdentifier, SqlCompoundOperator, SqlJoinType, SqlOrdering, SqlProjection},
    },
    Identifier,
};
use serde::{Deserialize, Serialize};

pub mod planner;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Aggregate {
    Average(Expr),
    Count(Expr),
    Max(Expr),
    Min(Expr),
    Sum(Expr),
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
        group_by: Vec<Expr>,
        aggregates: Vec<Aggregate>,
    },

    Filter {
        source: Box<Node>,
        predicate: Expr,
    },

    Projection {
        source: Box<Node>,
        fields: Vec<SqlProjection>
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
        key: Vec<(Expr, SqlOrdering)>,
    },

    Values {
        rows: Vec<Vec<Expr>>,
    },

    ValuesHandle {
        identifier: Identifier,
    },

    Scan {
        source: SqlCollectionIdentifier,
        filter: Option<Expr>,
    },

    Join {
        left: Box<Node>,
        join_type: SqlJoinType,
        right: Box<Node>,
        constraint: Option<Expr>,
    },

    Subquery {
        source: Box<Node>,
        alias: Identifier,
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
            Node::Order { source, key } => {
                let key_description = key
                    .iter()
                    .map(|(expr, ordering)| format!("({}, {:?})", expr, ordering))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{}- order [{}]{}", indent_str, key_description, Self::NEWLINE)?;
                source._fmt_recursive(f, indent + 1)
            }
            Node::Projection { source, fields } => {
                let fields_description = fields
                    .iter()
                    .map(|field|
                        match field {
                            SqlProjection::All { collection } => {
                                if let Some(c) = collection.as_ref() {
                                    return format!("* in {}", c.name);
                                }
                                return format!("*");
                            },
                            SqlProjection::Expr { expr, alias } => {
                                if let Some(alias) = alias {
                                    return format!("{} as {}", expr, alias.name);
                                }
                                return format!("{} as {}", expr, expr);
                            }
                        }
                    )
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{}- project [{}]{}", indent_str, fields_description, Self::NEWLINE)?;
                
                source._fmt_recursive(f, indent + 1)
            }
            Node::Filter { source, predicate } => {
                write!(f, "{}- filter [{}]{}", indent_str, predicate, Self::NEWLINE)?;
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
                write!(f, "{}- limit [count={}]{}", indent_str, limit, Self::NEWLINE)?;
                source._fmt_recursive(f, indent + 1)
            }
            Node::Offset { source, offset } => {
                write!(f, "{}- offset [count={}]{}", indent_str, offset, Self::NEWLINE)?;
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
                    constraint.as_ref().unwrap(),
                    Self::NEWLINE
                )?;
                left._fmt_recursive(f, indent + 1)?;
                right._fmt_recursive(f, indent + 1)
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
