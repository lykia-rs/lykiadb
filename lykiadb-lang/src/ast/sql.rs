use serde::{Deserialize, Serialize};

use super::{Identifier, expr::Expr};

// Enums

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub enum SqlDistinct {
    #[serde(rename = "SqlDistinct::ImplicitAll")]
    ImplicitAll,
    #[serde(rename = "SqlDistinct::All")]
    All,
    #[serde(rename = "SqlDistinct::Distinct")]
    Distinct,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub enum SqlJoinType {
    #[serde(rename = "SqlJoinType::Left")]
    Left,
    #[serde(rename = "SqlJoinType::Right")]
    Right,
    #[serde(rename = "SqlJoinType::Inner")]
    Inner,
    #[serde(rename = "SqlJoinType::Cross")]
    Cross,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
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

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub enum SqlOrdering {
    #[serde(rename = "SqlOrdering::Asc")]
    Asc,
    #[serde(rename = "SqlOrdering::Desc")]
    Desc,
}

//
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub struct SqlCollectionIdentifier {
    pub namespace: Option<Identifier>,
    pub name: Identifier,
    pub alias: Option<Identifier>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub enum SqlProjection {
    #[serde(rename = "SqlProjection::All")]
    All { collection: Option<Identifier> },
    #[serde(rename = "SqlProjection::Expr")]
    Expr {
        expr: Box<Expr>,
        alias: Option<Identifier>,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub struct SqlLimitClause {
    pub count: Box<Expr>,
    pub offset: Option<Box<Expr>>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub struct SqlOrderByClause {
    pub expr: Box<Expr>,
    pub ordering: SqlOrdering,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub struct SqlSelectCompound {
    pub operator: SqlCompoundOperator,
    pub core: SqlSelectCore,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub struct SqlExpressionSource {
    pub expr: Box<Expr>,
    pub alias: Identifier,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub enum SqlSource {
    Collection(SqlCollectionIdentifier),
    Expr(SqlExpressionSource),
}

impl SqlSource {
    pub fn alias(&self) -> &Identifier {
        match self {
            SqlSource::Collection(collection) => {
                collection.alias.as_ref().unwrap_or(&collection.name)
            }
            SqlSource::Expr(expr) => &expr.alias,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub enum SqlFrom {
    #[serde(rename = "SqlFrom::Source")]
    Source(SqlSource),
    #[serde(rename = "SqlFrom::Group")]
    Group { values: Vec<SqlFrom> },
    #[serde(rename = "SqlFrom::Select")]
    Select {
        subquery: Box<SqlSelect>,
        alias: Option<Identifier>,
    },
    #[serde(rename = "SqlFrom::Join")]
    Join {
        left: Box<SqlFrom>,
        join_type: SqlJoinType,
        right: Box<SqlFrom>,
        constraint: Option<Box<Expr>>,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub struct SqlSelectCore {
    pub distinct: SqlDistinct,
    pub projection: Vec<SqlProjection>,
    pub from: Option<SqlFrom>,
    pub r#where: Option<Box<Expr>>,
    pub group_by: Option<Vec<Expr>>,
    pub having: Option<Box<Expr>>,
    pub compound: Option<Box<SqlSelectCompound>>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub struct SqlSelect {
    pub core: SqlSelectCore,
    pub order_by: Option<Vec<SqlOrderByClause>>,
    pub limit: Option<SqlLimitClause>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub enum SqlValues {
    #[serde(rename = "SqlValues::Values")]
    Values { values: Vec<Expr> },
    #[serde(rename = "SqlValues::Select")]
    Select(Box<SqlSelect>),
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub struct SqlInsert {
    pub collection: SqlCollectionIdentifier,
    pub values: SqlValues,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub struct SqlUpdate {
    pub collection: SqlCollectionIdentifier,
    pub assignments: Vec<Expr>,
    pub r#where: Option<Box<Expr>>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "@type")]
pub struct SqlDelete {
    pub collection: SqlCollectionIdentifier,
    pub r#where: Option<Box<Expr>>,
}
