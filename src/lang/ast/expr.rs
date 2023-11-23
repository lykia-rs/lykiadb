use crate::{lang::token::Token, runtime::types::RV};

use super::sql::SqlSelect;

#[derive(Debug, Eq, PartialEq)]
pub enum Expr {
    Select(SqlSelect),
    Variable(Token),
    Grouping(ExprId),
    Literal(RV),
    Binary {
        left: ExprId,
        token: Token,
        right: ExprId,
    },
    Unary {
        token: Token,
        expr: ExprId,
    },
    Assignment {
        var_tok: Token, 
        expr: ExprId,
    },
    Logical {
        left: ExprId,
        token: Token,
        right: ExprId,
    },
    Call {
        callee: ExprId,
        paren: Token,
        args: Vec<ExprId> 
    },
}

pub type ExprId = usize;
