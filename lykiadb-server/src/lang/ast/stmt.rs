use serde::{Deserialize, Serialize};

use crate::lang::{
    tokenizer::token::{Span, Spanned},
    Identifier,
};

use super::expr::Expr;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub enum Stmt {
    #[serde(rename = "Stmt::Program")]
    Program {
        body: Vec<Stmt>,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::Expression")]
    Expression {
        expr: Box<Expr>,
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
        body: Vec<Stmt>,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::Declaration")]
    Declaration {
        dst: Identifier,
        expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::If")]
    If {
        condition: Box<Expr>,
        body: Box<Stmt>,
        r#else_body: Option<Box<Stmt>>,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::Loop")]
    Loop {
        condition: Option<Box<Expr>>,
        body: Box<Stmt>,
        post: Option<Box<Stmt>>,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Stmt::Return")]
    Return {
        expr: Option<Box<Expr>>,
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
