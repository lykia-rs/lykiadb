use crate::lang::Identifier;
use serde::{Deserialize, Serialize};

use super::expr::Expr;

// Enums

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub enum SqlDistinct {
    #[serde(rename = "SqlDistinct::ImplicitAll")]
    ImplicitAll,
    #[serde(rename = "SqlDistinct::All")]
    All,
    #[serde(rename = "SqlDistinct::Distinct")]
    Distinct,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
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

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
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

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub enum SqlOrdering {
    #[serde(rename = "SqlOrdering::Asc")]
    Asc,
    #[serde(rename = "SqlOrdering::Desc")]
    Desc,
}

//
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub struct SqlCollectionIdentifier {
    pub namespace: Option<Identifier>,
    pub name: Identifier,
    pub alias: Option<Identifier>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub enum SqlExpr {
    #[serde(rename = "SqlExpr::Default")]
    Default(Box<Expr>),
    #[serde(rename = "SqlExpr::Is")]
    Is {
        left: Box<SqlExpr>,
        right: Box<SqlExpr>,
    },
    #[serde(rename = "SqlExpr::IsNot")]
    IsNot {
        left: Box<SqlExpr>,
        right: Box<SqlExpr>,
    },
    #[serde(rename = "SqlExpr::In")]
    In {
        left: Box<SqlExpr>,
        right: Box<SqlExpr>,
    },
    #[serde(rename = "SqlExpr::NotIn")]
    NotIn {
        left: Box<SqlExpr>,
        right: Box<SqlExpr>,
    },
    #[serde(rename = "SqlExpr::Like")]
    Like {
        left: Box<SqlExpr>,
        right: Box<SqlExpr>,
    },
    #[serde(rename = "SqlExpr::NotLike")]
    NotLike {
        left: Box<SqlExpr>,
        right: Box<SqlExpr>,
    },
    #[serde(rename = "SqlExpr::Between")]
    Between {
        expr: Box<SqlExpr>,
        lower: Box<SqlExpr>,
        upper: Box<SqlExpr>,
    },
    #[serde(rename = "SqlExpr::NotBetween")]
    NotBetween {
        expr: Box<SqlExpr>,
        lower: Box<SqlExpr>,
        upper: Box<SqlExpr>,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub enum SqlProjection {
    #[serde(rename = "SqlProjection::All")]
    All { collection: Option<Identifier> },
    #[serde(rename = "SqlProjection::Expr")]
    Expr {
        expr: Box<SqlExpr>,
        alias: Option<Identifier>,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub struct SqlLimitClause {
    pub count: Box<SqlExpr>,
    pub offset: Option<Box<SqlExpr>>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub struct SqlOrderByClause {
    pub expr: Box<SqlExpr>,
    pub ordering: SqlOrdering,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub struct SqlSelectCompound {
    pub operator: SqlCompoundOperator,
    pub core: SqlSelectCore,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub enum SqlCollectionSubquery {
    #[serde(rename = "SqlCollectionSubquery::Group")]
    Group { values: Vec<SqlCollectionSubquery> },
    #[serde(rename = "SqlCollectionSubquery::Collection")]
    Collection(SqlCollectionIdentifier),
    #[serde(rename = "SqlCollectionSubquery::Select")]
    Select {
        expr: Box<Expr>,
        alias: Option<Identifier>,
    },
    #[serde(rename = "SqlCollectionSubquery::Join")]
    Join {
        left: Box<SqlCollectionSubquery>,
        join_type: SqlJoinType,
        right: Box<SqlCollectionSubquery>,
        constraint: Option<Box<SqlExpr>>,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub struct SqlSelectCore {
    pub distinct: SqlDistinct,
    pub projection: Vec<SqlProjection>,
    pub from: Option<SqlCollectionSubquery>,
    pub r#where: Option<Box<SqlExpr>>,
    pub group_by: Option<Vec<SqlExpr>>,
    pub having: Option<Box<SqlExpr>>,
    pub compound: Option<Box<SqlSelectCompound>>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub struct SqlSelect {
    pub core: SqlSelectCore,
    pub order_by: Option<Vec<SqlOrderByClause>>,
    pub limit: Option<SqlLimitClause>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub enum SqlValues {
    #[serde(rename = "SqlValues::Values")]
    Values { values: Vec<SqlExpr> },
    #[serde(rename = "SqlValues::Select")]
    Select(SqlSelect),
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub struct SqlInsert {
    pub collection: SqlCollectionIdentifier,
    pub values: SqlValues,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub struct SqlUpdate {
    pub collection: SqlCollectionIdentifier,
    pub assignments: Vec<SqlExpr>,
    pub r#where: Option<Box<SqlExpr>>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub struct SqlDelete {
    pub collection: SqlCollectionIdentifier,
    pub r#where: Option<Box<SqlExpr>>,
}
