use crate::{lang::token::Token, runtime::types::RV};

use super::sql::SqlSelect;

#[derive(Debug, Eq, PartialEq)]
pub enum Expr {
    Select(SqlSelect),
    Binary {
        left: ExprId,
        token: Token,
        right: ExprId,
    },
    Grouping(ExprId),
    Literal(RV),
    Unary {
        token: Token,
        expr: ExprId,
    },
    Variable(Token),
    Assignment(Token, ExprId),
    Logical {
        left: ExprId,
        token: Token,
        right: ExprId,
    },
    Call(ExprId, Token, Vec<ExprId>),
}

pub type ExprId = usize;
