use lykiadb_lang::{ast::sql::{SqlCollectionIdentifier, SqlExpr, SqlJoinType, SqlOrdering}, Identifier};
use serde::{Deserialize, Serialize};

pub mod planner;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Aggregate {
    Average(SqlExpr),
    Count(SqlExpr),
    Max(SqlExpr),
    Min(SqlExpr),
    Sum(SqlExpr),
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Plan {
    Select(Node),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Aggregate {
        source: Box<Node>,
        group_by: Vec<SqlExpr>,
        aggregates: Vec<Aggregate>,
    },

    Filter {
        source: Box<Node>,
        predicate: SqlExpr,
    },

    Projection {
        source: Box<Node>,
        expressions: Vec<SqlExpr>,
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
        key: Vec<(SqlExpr, SqlOrdering)>,
    },

    Values {
        rows: Vec<Vec<SqlExpr>>,
    },

    ValuesHandle {
        identifier: Identifier
    },

    Scan {
        source: SqlCollectionIdentifier,
        filter: Option<SqlExpr>,
    },

    Join {
        left: Box<Node>,
        join_type: SqlJoinType,
        right: Box<Node>,
        constraint: Option<SqlExpr>,
    },

    Subquery {
        source: Box<Node>,
        alias: Identifier,
    },

    Nothing
}
