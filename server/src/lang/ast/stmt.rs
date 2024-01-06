use serde::Serialize;

use crate::lang::token::{Span, Spanned, Token};

use super::expr::ExprId;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum Stmt {
    Program {
        body: Vec<StmtId>,
        #[serde(skip)]
        span: Span,
    },
    Expression {
        expr: ExprId,
        #[serde(skip)]
        span: Span,
    },
    Break {
        #[serde(skip)]
        span: Span,
    },
    Continue {
        #[serde(skip)]
        span: Span,
    },
    Block {
        body: Vec<StmtId>,
        #[serde(skip)]
        span: Span,
    },
    Declaration {
        dst: Token,
        expr: ExprId,
        #[serde(skip)]
        span: Span,
    },
    If {
        condition: ExprId,
        body: StmtId,
        r#else_body: Option<StmtId>,
        #[serde(skip)]
        span: Span,
    },
    Loop {
        condition: Option<ExprId>,
        body: StmtId,
        post: Option<StmtId>,
        #[serde(skip)]
        span: Span,
    },
    Return {
        expr: Option<ExprId>,
        #[serde(skip)]
        span: Span,
    },
}

impl Spanned for Stmt {
    fn get_span(&self) -> Span {
        match self {
            Stmt::Program { span, .. } => *span,
            Stmt::Expression { span, .. } => *span,
            Stmt::Break { span, .. } => *span,
            Stmt::Continue { span, .. } => *span,
            Stmt::Block { span, .. } => *span,
            Stmt::Declaration { span, .. } => *span,
            Stmt::If { span, .. } => *span,
            Stmt::Loop { span, .. } => *span,
            Stmt::Return { span, .. } => *span,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Copy)] 
pub struct StmtId(pub usize);
