use serde::ser::SerializeMap;
use serde::Serialize;

use crate::lang::{
    tokenizer::token::{Span, Spanned},
    Identifier,
};

use super::{expr::ExprId, AstRef};

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
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

pub type StmtId = AstRef<Stmt>;

pub const STMT_ID_PLACEHOLDER: &'static str = "@StmtId";

impl Serialize for AstRef<Stmt> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(STMT_ID_PLACEHOLDER, &self.0)?;
        map.end()
    }
}
