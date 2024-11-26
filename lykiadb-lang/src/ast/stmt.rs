use serde::{Deserialize, Serialize};
use derivative::Derivative;

use crate::{Identifier, Span, Spanned};

use super::expr::Expr;

#[derive(Debug, Serialize, Deserialize, Clone, Derivative)]
#[serde(tag = "@type")]
#[derivative(Eq, PartialEq)]
pub enum Stmt {
    #[serde(rename = "Stmt::Program")]
    Program {
        body: Vec<Stmt>,
        #[serde(skip)]
#[derivative(PartialEq="ignore")]

        span: Span,
    },
    #[serde(rename = "Stmt::Expression")]
    Expression {
        expr: Box<Expr>,
        #[serde(skip)]
#[derivative(PartialEq="ignore")]

        span: Span,
    },
    #[serde(rename = "Stmt::Break")]
    Break {
        #[serde(skip)]
#[derivative(PartialEq="ignore")]

        span: Span,
    },
    #[serde(rename = "Stmt::Continue")]
    Continue {
        #[serde(skip)]
#[derivative(PartialEq="ignore")]

        span: Span,
    },
    #[serde(rename = "Stmt::Block")]
    Block {
        body: Vec<Stmt>,
        #[serde(skip)]
#[derivative(PartialEq="ignore")]

        span: Span,
    },
    #[serde(rename = "Stmt::Declaration")]
    Declaration {
        dst: Identifier,
        expr: Box<Expr>,
        #[serde(skip)]
#[derivative(PartialEq="ignore")]

        span: Span,
    },
    #[serde(rename = "Stmt::If")]
    If {
        condition: Box<Expr>,
        body: Box<Stmt>,
        r#else_body: Option<Box<Stmt>>,
        #[serde(skip)]
#[derivative(PartialEq="ignore")]

        span: Span,
    },
    #[serde(rename = "Stmt::Loop")]
    Loop {
        condition: Option<Box<Expr>>,
        body: Box<Stmt>,
        post: Option<Box<Stmt>>,
        #[serde(skip)]
#[derivative(PartialEq="ignore")]

        span: Span,
    },
    #[serde(rename = "Stmt::Return")]
    Return {
        expr: Option<Box<Expr>>,
        #[serde(skip)]
#[derivative(PartialEq="ignore")]

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
