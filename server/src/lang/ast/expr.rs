use std::rc::Rc;
use serde::Serialize;

use super::{sql::{SqlSelect, SqlInsert, SqlDelete, SqlUpdate}, stmt::StmtId, Literal};
use crate::lang::token::{Span, Spanned, Token};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    IsEqual,
    IsNotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,
    Not,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Expr {
    #[serde(rename = "Expr::Select")]
    Select {
        query: SqlSelect,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Expr::Insert")]
    Insert {
        command: SqlInsert,
        #[serde(skip)]
        span: Span
    },
    #[serde(rename = "Expr::Update")]
    Update {
        command: SqlUpdate,
        #[serde(skip)]
        span: Span
    },
    #[serde(rename = "Expr::Delete")]
    Delete {
        command: SqlDelete,
        #[serde(skip)]
        span: Span
    },
    #[serde(rename = "Expr::Variable")]
    Variable {
        name: Token,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Expr::Grouping")]
    Grouping {
        expr: ExprId,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Expr::Literal")]
    Literal {
        value: Literal,
        raw: String,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Expr::Function")]
    Function {
        name: Option<Token>,
        parameters: Vec<Token>,
        body: Rc<Vec<StmtId>>,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Expr::Binary")]
    Binary {
        left: ExprId,
        operation: Operation,
        right: ExprId,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Expr::Unary")]
    Unary {
        operation: Operation,
        expr: ExprId,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Expr::Assignment")]
    Assignment {
        dst: Token,
        expr: ExprId,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Expr::Logical")]
    Logical {
        left: ExprId,
        operation: Operation,
        right: ExprId,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Expr::Call")]
    Call {
        callee: ExprId,
        args: Vec<ExprId>,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Expr::Get")]
    Get {
        object: ExprId,
        name: Token,
        #[serde(skip)]
        span: Span,
    },
    #[serde(rename = "Expr::Set")]
    Set {
        object: ExprId,
        name: Token,
        value: ExprId,
        #[serde(skip)]
        span: Span,
    },
}

impl Spanned for Expr {
    fn get_span(&self) -> Span {
        match self {
            Expr::Select { query: _, span }
            | Expr::Insert { command: _, span }
            | Expr::Delete { command: _, span }
            | Expr::Update { command: _, span }
            | Expr::Variable { name: _, span }
            | Expr::Grouping { expr: _, span }
            | Expr::Literal {
                value: _,
                raw: _,
                span,
            }
            | Expr::Function {
                name: _,
                parameters: _,
                body: _,
                span,
            }
            | Expr::Binary {
                left: _,
                operation: _,
                right: _,
                span,
            }
            | Expr::Unary {
                operation: _,
                expr: _,
                span,
            }
            | Expr::Assignment {
                dst: _,
                expr: _,
                span,
            }
            | Expr::Logical {
                left: _,
                operation: _,
                right: _,
                span,
            }
            | Expr::Call {
                callee: _,
                args: _,
                span,
            }
            | Expr::Get {
                object: _,
                name: _,
                span,
            }
            | Expr::Set {
                object: _,
                name: _,
                value: _,
                span,
            } => *span,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct ExprId(pub usize);

