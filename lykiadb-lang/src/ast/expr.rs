use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    tokenizer::token::{Span, Spanned},
    Identifier,
};

use super::{
    sql::{SqlDelete, SqlInsert, SqlSelect, SqlUpdate},
    stmt::Stmt, AstNode,
};

use crate::Literal;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@type")]
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

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub enum Expr {
    #[serde(rename = "Expr::Select")]
    Select {
        query: SqlSelect,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Insert")]
    Insert {
        command: SqlInsert,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Update")]
    Update {
        command: SqlUpdate,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Delete")]
    Delete {
        command: SqlDelete,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Variable")]
    Variable {
        name: Identifier,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Grouping")]
    Grouping {
        expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Literal")]
    Literal {
        value: Literal,
        raw: String,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Function")]
    Function {
        name: Option<Identifier>,
        parameters: Vec<Identifier>,
        body: Arc<Vec<Stmt>>,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Binary")]
    Binary {
        left: Box<Expr>,
        operation: Operation,
        right: Box<Expr>,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Unary")]
    Unary {
        operation: Operation,
        expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Assignment")]
    Assignment {
        dst: Identifier,
        expr: Box<Expr>,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Logical")]
    Logical {
        left: Box<Expr>,
        operation: Operation,
        right: Box<Expr>,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Call")]
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Get")]
    Get {
        object: Box<Expr>,
        name: Identifier,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
    #[serde(rename = "Expr::Set")]
    Set {
        object: Box<Expr>,
        name: Identifier,
        value: Box<Expr>,
        #[serde(skip)]
        span: Span,
        #[serde(skip)]
        id: usize,
    },
}

impl Spanned for Expr {
    fn get_span(&self) -> Span {
        match self {
            Expr::Select { query: _, span,
                id: _, }
            | Expr::Insert { command: _, span,
                id: _, }
            | Expr::Delete { command: _, span,
                id: _, }
            | Expr::Update { command: _, span,
                id: _, }
            | Expr::Variable {
                name: _,
                span,
                id: _,
            }
            | Expr::Grouping { expr: _, span,
                id: _, }
            | Expr::Literal {
                value: _,
                raw: _,
                span,
                id: _,
            }
            | Expr::Function {
                name: _,
                parameters: _,
                body: _,
                span,
                id: _,
            }
            | Expr::Binary {
                left: _,
                operation: _,
                right: _,
                span,
                id: _,
            }
            | Expr::Unary {
                operation: _,
                expr: _,
                span,
                id: _,
            }
            | Expr::Assignment {
                dst: _,
                expr: _,
                span,
                id: _,
            }
            | Expr::Logical {
                left: _,
                operation: _,
                right: _,
                span,
                id: _,
            }
            | Expr::Call {
                callee: _,
                args: _,
                span,
                id: _,
            }
            | Expr::Get {
                object: _,
                name: _,
                span,
                id: _,
            }
            | Expr::Set {
                object: _,
                name: _,
                value: _,
                span,
                id: _,
            } => *span,
        }
    }
}

impl AstNode for Expr {
    fn get_id(&self) -> usize {
        match self {
            Expr::Select { query: _, span,
                id, }
            | Expr::Insert { command: _, span,
                id, }
            | Expr::Delete { command: _, span,
                id, }
            | Expr::Update { command: _, span,
                id, }
            | Expr::Variable {
                name: _,
                span,
                id,
            }
            | Expr::Grouping { expr: _, span,
                id, }
            | Expr::Literal {
                value: _,
                raw: _,
                span,
                id,
            }
            | Expr::Function {
                name: _,
                parameters: _,
                body: _,
                span,
                id,
            }
            | Expr::Binary {
                left: _,
                operation: _,
                right: _,
                span,
                id,
            }
            | Expr::Unary {
                operation: _,
                expr: _,
                span,
                id,
            }
            | Expr::Assignment {
                dst: _,
                expr: _,
                span,
                id,
            }
            | Expr::Logical {
                left: _,
                operation: _,
                right: _,
                span,
                id,
            }
            | Expr::Call {
                callee: _,
                args: _,
                span,
                id,
            }
            | Expr::Get {
                object: _,
                name: _,
                span,
                id,
            }
            | Expr::Set {
                object: _,
                name: _,
                value: _,
                span,
                id,
            } => *id,
        }
    }
}