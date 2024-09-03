use lykiadb_lang::{
    ast::{
        expr::Expr,
        sql::{SqlCollectionIdentifier, SqlJoinType, SqlOrdering},
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
        expressions: Vec<Expr>,
        aliases: Vec<String>,
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
