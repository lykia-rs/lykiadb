use std::rc::Rc;

use serde::{Deserialize, Serialize};

use super::{sql::SqlSelect, stmt::StmtId, Literal};
use crate::lang::token::{Keyword, Span, Spanned, Symbol, Token, TokenType};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
        value: Literal,
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
        operation: Operation,
        right: ExprId,
        span: Span,
    },
    Unary {
        operation: Operation,
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
        operation: Operation,
        right: ExprId,
        span: Span,
    },
    Call {
        callee: ExprId,
        args: Vec<ExprId>,
        span: Span,
    },
    Get {
        object: ExprId,
        name: Token,
        span: Span,
    },
    Set {
        object: ExprId,
        name: Token,
        value: ExprId,
        span: Span,
    },
}

impl Spanned for Expr {
    fn get_span(&self) -> Span {
        match self {
            Expr::Select { query: _, span }
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
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ExprId(pub usize);
