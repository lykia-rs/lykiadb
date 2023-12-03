use crate::lang::token::Token;

use super::expr::ExprId;

#[derive(Debug, Eq, PartialEq)]
pub enum Stmt {
    Expression(ExprId),
    Break(Token),
    Continue(Token),
    Block(Vec<StmtId>),
    Declaration {
        token: Token,
        expr: ExprId,
    },
    If {
        condition: ExprId,
        body: StmtId,
        r#else: Option<StmtId>,
    },
    Loop {
        condition: Option<ExprId>,
        body: StmtId,
        post: Option<StmtId>,
    },
    Return {
        token: Token,
        expr: Option<ExprId>,
    },
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StmtId(pub usize);
