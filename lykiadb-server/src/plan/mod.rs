use lykiadb_lang::ast::sql::SqlExpr;
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
pub enum Direction {
    Ascending,
    Descending,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Plan {
    Select(Node)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Aggregate { source: Box<Node>, group_by: Vec<SqlExpr>, aggregates: Vec<Aggregate> },

    Filter { source: Box<Node>, predicate: SqlExpr },

    Projection { source: Box<Node>, expressions: Vec<SqlExpr>, aliases: Vec<String> },
    
    Scan { collection: String, filter: Option<SqlExpr>, alias: Option<String> },

    Limit { source: Box<Node>, limit: usize },

    Offset { source: Box<Node>, offset: usize },

    Order { source: Box<Node>, key: Vec<(SqlExpr, Direction)> },

    Values { rows: Vec<Vec<SqlExpr>> },
}