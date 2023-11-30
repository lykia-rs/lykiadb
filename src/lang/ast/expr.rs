use std::rc::Rc;

use crate::{lang::token::Token, runtime::types::RV};

use super::{sql::SqlSelect, stmt::StmtId};

#[derive(Debug, Eq, PartialEq)]
pub enum Expr {
    Select(SqlSelect),
    Variable(Token),
    Grouping(ExprId),
    Literal(RV),
    Function(Token, Vec<Token>, Rc<Vec<StmtId>>),
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
        args: Vec<ExprId>,
    },
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ExprId(pub usize);
