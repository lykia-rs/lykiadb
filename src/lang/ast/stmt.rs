use crate::lang::token::{Span, Token};

use super::expr::ExprId;

#[derive(Debug, Eq, PartialEq)]
pub enum Stmt {
    Program {
        stmts: Vec<StmtId>,
        span: Span,
    },
    Expression {
        expr: ExprId,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Block {
        stmts: Vec<StmtId>,
        span: Span,
    },
    Declaration {
        dst: Token,
        expr: ExprId,
        span: Span,
    },
    If {
        condition: ExprId,
        body: StmtId,
        r#else: Option<StmtId>,
        span: Span,
    },
    Loop {
        condition: Option<ExprId>,
        body: StmtId,
        post: Option<StmtId>,
        span: Span,
    },
    Return {
        expr: Option<ExprId>,
        span: Span,
    },
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StmtId(pub usize);
