use crate::{lang::token::Token, runtime::types::RV};

use super::sql::SqlSelect;

#[derive(Debug, Eq, PartialEq)]
pub enum Expr {
    Select(SqlSelect),
    Binary(Token, ExprId, ExprId),
    Grouping(ExprId),
    Literal(RV),
    Unary(Token, ExprId),
    Variable(Token),
    Assignment(Token, ExprId),
    Logical(ExprId, Token, ExprId),
    Call(ExprId, Token, Vec<ExprId>),
}

pub type ExprId = usize;
