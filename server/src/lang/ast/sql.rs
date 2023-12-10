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
pub enum SqlFrom {
    CollectionSubquery(Vec<SqlCollectionSubquery>),
    JoinClause(Box<SqlJoinClause>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlProjection {
    All,
    /*AllFieldsOf{
        collection: Token
    },*/
    Complex { expr: SqlExpr, alias: Option<Token> },
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlJoinClause {
    None(SqlCollectionSubquery),
    Join(Vec<(SqlJoinType, SqlCollectionSubquery, SqlExpr)>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlCollectionSubquery {
    Simple {
        namespace: Option<Token>,
        collection: Token,
        alias: Option<Token>,
    },
    From {
        from: SqlFrom,
    },
    Select {
        stmt: SqlSelect,
        alias: Option<Token>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct SelectCore {
    pub distinct: SqlDistinct,
    pub projection: Vec<SqlProjection>,
    pub from: Option<SqlFrom>,
    pub r#where: Option<SqlExpr>,
    pub group_by: Option<Vec<SqlExpr>>,
    pub having: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SqlSelect {
    pub core: SelectCore,
    pub compound: Vec<(SqlCompoundOperator, SelectCore)>,
    pub order_by: Option<Vec<(SqlExpr, SqlOrdering)>>,
    pub limit: Option<(SqlExpr, Option<SqlExpr>)>,
}
