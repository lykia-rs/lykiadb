use crate::lang::token::Token;
use serde::Serialize;

use super::expr::ExprId;

// Enums

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum SqlDistinct {
    ImplicitAll,
    All,
    Distinct,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum SqlJoinType {
    Left,
    LeftOuter,
    Right,
    Inner,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum SqlCompoundOperator {
    Union,
    UnionAll,
    Intersect,
    Except,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum SqlOrdering {
    Asc,
    Desc,
}

//
#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct SqlCollectionIdentifier {
    pub namespace: Option<Token>,
    pub name: Token,
    pub alias: Option<Token>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum SqlExpr {
    Default(ExprId),
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum SqlProjection {
    All { collection: Option<Token> },
    Expr { expr: SqlExpr, alias: Option<Token> },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct SqlLimitClause {
    pub count: SqlExpr,
    pub offset: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct SqlOrderByClause {
    pub expr: SqlExpr,
    pub ordering: SqlOrdering,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct SqlSelectCompound {
    pub operator: SqlCompoundOperator,
    pub core: SqlSelectCore,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct SqlJoin {
    pub join_type: SqlJoinType,
    pub subquery: SqlCollectionSubquery,
    pub join_constraint: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum SqlCollectionSubquery {
    Group(Vec<SqlCollectionSubquery>),
    Join(Box<SqlCollectionSubquery>, Vec<SqlJoin>),
    Collection(SqlCollectionIdentifier),
    Select {
        expr: ExprId,
        alias: Option<Token>,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct SqlSelectCore {
    pub distinct: SqlDistinct,
    pub projection: Vec<SqlProjection>,
    pub from: Option<SqlCollectionSubquery>,
    pub r#where: Option<SqlExpr>,
    pub group_by: Option<Vec<SqlExpr>>,
    pub having: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct SqlSelect {
    pub core: SqlSelectCore,
    pub compound: Vec<SqlSelectCompound>,
    pub order_by: Option<Vec<SqlOrderByClause>>,
    pub limit: Option<SqlLimitClause>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum SqlValues {
    Values(Vec<SqlExpr>),
    Select(SqlSelect)
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct SqlInsert {
    pub collection: SqlCollectionIdentifier,
    pub values: SqlValues,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct SqlUpdate {
    pub collection: SqlCollectionIdentifier,
    pub assignments: Vec<SqlExpr>,
    pub r#where: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct SqlDelete {
    pub collection: SqlCollectionIdentifier,
    pub r#where: Option<SqlExpr>,
}