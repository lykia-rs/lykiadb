use std::rc::Rc;

use crate::{
    lang::token::{Span, Token, TokenType},
    runtime::types::RV,
};

use super::{sql::SqlSelect, stmt::StmtId};

#[derive(Debug, Eq, PartialEq)]
pub enum Expr {
    Select {
        query: SqlSelect,
        span: Span,
    },
    Variable {
        name: Token,
        span: Span,
    },
    Grouping {
        expr: ExprId,
        span: Span,
    },
    Literal {
        value: RV,
        raw: String,
        span: Span,
    },
    Function {
        name: Option<Token>,
        parameters: Vec<Token>,
        body: Rc<Vec<StmtId>>,
        span: Span,
    },
    Binary {
        left: ExprId,
        symbol: TokenType,
        right: ExprId,
        span: Span,
    },
    Unary {
        symbol: TokenType,
        expr: ExprId,
        span: Span,
    },
    Assignment {
        dst: Token,
        expr: ExprId,
        span: Span,
    },
    Logical {
        left: ExprId,
        symbol: TokenType,
        right: ExprId,
        span: Span,
    },
    Call {
        callee: ExprId,
        args: Vec<ExprId>,
        span: Span,
    },
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ExprId(pub usize);
