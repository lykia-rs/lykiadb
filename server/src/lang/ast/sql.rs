use crate::lang::token::Token;

use super::expr::ExprId;

// Enums

#[derive(Debug, Eq, PartialEq)]
pub enum SqlDistinct {
    All,
    Distinct,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlJoinType {
    Left,
    LeftOuter,
    Right,
    Inner,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlCompoundOperator {
    Union,
    UnionAll,
    Intersect,
    Except,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlOrdering {
    Asc,
    Desc,
}

//

#[derive(Debug, Eq, PartialEq)]
pub enum SqlExpr {
    Default(ExprId),
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlProjection {
    All { collection: Option<Token> },
    Expr { expr: SqlExpr, alias: Option<Token> },
}

#[derive(Debug, Eq, PartialEq)]
pub struct SqlLimitClause {
    pub count: SqlExpr,
    pub offset: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SqlOrderByClause {
    pub expr: SqlExpr,
    pub ordering: SqlOrdering,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SqlSelectCompound {
    pub operator: SqlCompoundOperator,
    pub core: SqlSelectCore,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SqlJoin {
    pub join_type: SqlJoinType,
    pub subquery: SqlCollectionSubquery,
    pub join_constraint: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlCollectionSubquery {
    Group(Vec<SqlCollectionSubquery>),
    Join(Box<SqlCollectionSubquery>, Vec<SqlJoin>),
    Collection {
        namespace: Option<Token>,
        name: Token,
        alias: Option<Token>,
    },
    Select {
        expr: ExprId,
        alias: Option<Token>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct SqlSelectCore {
    pub distinct: SqlDistinct,
    pub projection: Vec<SqlProjection>,
    pub from: Option<SqlCollectionSubquery>,
    pub r#where: Option<SqlExpr>,
    pub group_by: Option<Vec<SqlExpr>>,
    pub having: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SqlSelect {
    pub core: SqlSelectCore,
    pub compound: Vec<SqlSelectCompound>,
    pub order_by: Option<Vec<SqlOrderByClause>>,
    pub limit: Option<SqlLimitClause>,
}
