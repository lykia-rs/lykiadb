use crate::lang::token::Token;

use super::expr::ExprId;

#[derive(Debug, Eq, PartialEq)]
pub enum Stmt {
    Expression(ExprId),
    Declaration(Token, ExprId),
    Block(Vec<StmtId>),
    If(ExprId, StmtId, Option<StmtId>),
    Loop(Option<ExprId>, StmtId, Option<StmtId>),
    Break(Token),
    Continue(Token),
    Return(Token, Option<ExprId>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StmtId(pub usize);
