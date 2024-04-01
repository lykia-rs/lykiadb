use crate::lang::Identifier;
use serde::ser::SerializeMap;
use serde::Serialize;

use super::{expr::ExprId, AstRef};

// Enums

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub enum SqlDistinct {
    #[serde(rename = "SqlDistinct::ImplicitAll")]
    ImplicitAll,
    #[serde(rename = "SqlDistinct::All")]
    All,
    #[serde(rename = "SqlDistinct::Distinct")]
    Distinct,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
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

#[derive(Debug, Eq, PartialEq, Serialize)]
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

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub enum SqlOrdering {
    #[serde(rename = "SqlOrdering::Asc")]
    Asc,
    #[serde(rename = "SqlOrdering::Desc")]
    Desc,
}

//
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub struct SqlCollectionIdentifier {
    pub namespace: Option<Identifier>,
    pub name: Identifier,
    pub alias: Option<Identifier>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub enum SqlExpr {
    #[serde(rename = "SqlExpr::Default")]
    Default(ExprId),
    #[serde(rename = "SqlExpr::Is")]
    Is { left: SqlExprId, right: SqlExprId },
    #[serde(rename = "SqlExpr::IsNot")]
    IsNot { left: SqlExprId, right: SqlExprId },
    #[serde(rename = "SqlExpr::In")]
    In { left: SqlExprId, right: SqlExprId },
    #[serde(rename = "SqlExpr::NotIn")]
    NotIn { left: SqlExprId, right: SqlExprId },
    #[serde(rename = "SqlExpr::Like")]
    Like { left: SqlExprId, right: SqlExprId },
    #[serde(rename = "SqlExpr::NotLike")]
    NotLike { left: SqlExprId, right: SqlExprId },
    #[serde(rename = "SqlExpr::Between")]
    Between {
        expr: SqlExprId,
        lower: SqlExprId,
        upper: SqlExprId,
    },
    #[serde(rename = "SqlExpr::NotBetween")]
    NotBetween {
        expr: SqlExprId,
        lower: SqlExprId,
        upper: SqlExprId,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub enum SqlProjection {
    #[serde(rename = "SqlProjection::All")]
    All { collection: Option<Identifier> },
    #[serde(rename = "SqlProjection::Expr")]
    Expr {
        expr: SqlExprId,
        alias: Option<Identifier>,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub struct SqlLimitClause {
    pub count: SqlExprId,
    pub offset: Option<SqlExprId>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub struct SqlOrderByClause {
    pub expr: SqlExprId,
    pub ordering: SqlOrdering,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub struct SqlSelectCompound {
    pub operator: SqlCompoundOperator,
    pub core: SqlSelectCore,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub enum SqlCollectionSubquery {
    #[serde(rename = "SqlCollectionSubquery::Group")]
    Group { values: Vec<SqlCollectionSubquery> },
    #[serde(rename = "SqlCollectionSubquery::Collection")]
    Collection(SqlCollectionIdentifier),
    #[serde(rename = "SqlCollectionSubquery::Select")]
    Select {
        expr: ExprId,
        alias: Option<Identifier>,
    },
    #[serde(rename = "SqlCollectionSubquery::Join")]
    Join {
        left: Box<SqlCollectionSubquery>,
        join_type: SqlJoinType,
        right: Box<SqlCollectionSubquery>,
        constraint: Option<SqlExprId>,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub struct SqlSelectCore {
    pub distinct: SqlDistinct,
    pub projection: Vec<SqlProjection>,
    pub from: Option<SqlCollectionSubquery>,
    pub r#where: Option<SqlExprId>,
    pub group_by: Option<Vec<SqlExprId>>,
    pub having: Option<SqlExprId>,
    pub compound: Option<Box<SqlSelectCompound>>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub struct SqlSelect {
    pub core: SqlSelectCore,
    pub order_by: Option<Vec<SqlOrderByClause>>,
    pub limit: Option<SqlLimitClause>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub enum SqlValues {
    #[serde(rename = "SqlValues::Values")]
    Values { values: Vec<SqlExprId> },
    #[serde(rename = "SqlValues::Select")]
    Select(SqlSelect),
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub struct SqlInsert {
    pub collection: SqlCollectionIdentifier,
    pub values: SqlValues,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub struct SqlUpdate {
    pub collection: SqlCollectionIdentifier,
    pub assignments: Vec<SqlExprId>,
    pub r#where: Option<SqlExprId>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub struct SqlDelete {
    pub collection: SqlCollectionIdentifier,
    pub r#where: Option<SqlExprId>,
}

pub type SqlExprId = AstRef<SqlExpr>;

pub const SQL_EXPR_ID_PLACEHOLDER: &'static str = "@SqlExprId";

impl Serialize for AstRef<SqlExpr> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(SQL_EXPR_ID_PLACEHOLDER, &self.0)?;
        map.end()
    }
}
