use crate::lang::token::Token;
use serde::Serialize;

use super::expr::ExprId;

// Enums

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SqlDistinct {
    #[serde(rename = "SqlDistinct::ImplicitAll")]
    ImplicitAll,
    #[serde(rename = "SqlDistinct::All")]
    All,
    #[serde(rename = "SqlDistinct::Distinct")]
    Distinct,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SqlJoinType {
    #[serde(rename = "SqlJoinType::Left")]
    Left,
    #[serde(rename = "SqlJoinType::LeftOuter")]
    LeftOuter,
    #[serde(rename = "SqlJoinType::Right")]
    Right,
    #[serde(rename = "SqlJoinType::Inner")]
    Inner,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SqlCompoundOperator {
    #[serde(rename = "SqlCompoundOperator::Union")]
    Union,
    #[serde(rename = "SqlCompoundOperator::UnionAll")]
    UnionAll,
    #[serde(rename = "SqlCompoundOperator::Intersect")]
    Intersect,
    #[serde(rename = "SqlCompoundOperator::Except")]
    Except,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SqlOrdering {
    #[serde(rename = "SqlOrdering::Asc")]
    Asc,
    #[serde(rename = "SqlOrdering::Desc")]
    Desc,
}

//
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub struct SqlCollectionIdentifier {
    pub namespace: Option<Token>,
    pub name: Token,
    pub alias: Option<Token>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SqlExpr {
    #[serde(rename = "SqlExpr::Default")]
    Default(ExprId),
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SqlProjection {
    #[serde(rename = "SqlProjection::All")]
    All { collection: Option<Token> },
    #[serde(rename = "SqlProjection::Expr")]
    Expr { expr: SqlExpr, alias: Option<Token> },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub struct SqlLimitClause {
    pub count: SqlExpr,
    pub offset: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub struct SqlOrderByClause {
    pub expr: SqlExpr,
    pub ordering: SqlOrdering,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub struct SqlSelectCompound {
    pub operator: SqlCompoundOperator,
    pub core: SqlSelectCore,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub struct SqlJoin {
    pub join_type: SqlJoinType,
    pub subquery: SqlCollectionSubquery,
    pub join_constraint: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SqlCollectionSubquery {
    #[serde(rename = "SqlCollectionSubquery::Group")]
    Group {
        values: Vec<SqlCollectionSubquery>
    },
    #[serde(rename = "SqlCollectionSubquery::Join")]
    Join {
        query: Box<SqlCollectionSubquery>,
        joins: Vec<SqlJoin>
    },
    #[serde(rename = "SqlCollectionSubquery::Collection")]
    Collection(SqlCollectionIdentifier),
    #[serde(rename = "SqlCollectionSubquery::Select")]
    Select {
        expr: ExprId,
        alias: Option<Token>,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub struct SqlSelectCore {
    pub distinct: SqlDistinct,
    pub projection: Vec<SqlProjection>,
    pub from: Option<SqlCollectionSubquery>,
    pub r#where: Option<SqlExpr>,
    pub group_by: Option<Vec<SqlExpr>>,
    pub having: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub struct SqlSelect {
    pub core: SqlSelectCore,
    pub compound: Vec<SqlSelectCompound>,
    pub order_by: Option<Vec<SqlOrderByClause>>,
    pub limit: Option<SqlLimitClause>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SqlValues {
    #[serde(rename = "SqlValues::Values")]
    Values {
        values: Vec<SqlExpr>
    },
    #[serde(rename = "SqlValues::Select")]
    Select(SqlSelect)
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub struct SqlInsert {
    pub collection: SqlCollectionIdentifier,
    pub values: SqlValues,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub struct SqlUpdate {
    pub collection: SqlCollectionIdentifier,
    pub assignments: Vec<SqlExpr>,
    pub r#where: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub struct SqlDelete {
    pub collection: SqlCollectionIdentifier,
    pub r#where: Option<SqlExpr>,
}