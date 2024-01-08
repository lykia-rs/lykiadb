use serde::Serialize;

use crate::lang::{tokens::token::{Span, Spanned}, Identifier};

use super::expr::ExprId;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Stmt {
    #[serde(rename = "Stmt::Program")]
    Program {
        body: Vec<StmtId>,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::Expression")]
    Expression {
        expr: ExprId,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::Break")]
    Break {
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::Continue")]
    Continue {
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::Block")]
    Block {
        body: Vec<StmtId>,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::Declaration")]
    Declaration {
        dst: Identifier,
        expr: ExprId,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::If")]
    If {
        condition: ExprId,
        body: StmtId,
        r#else_body: Option<StmtId>,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::Loop")]
    Loop {
        condition: Option<ExprId>,
        body: StmtId,
        post: Option<StmtId>,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::Return")]
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
