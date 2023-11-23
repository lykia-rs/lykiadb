use std::rc::Rc;

use crate::lang::token::Token;

use super::expr::ExprId;

#[derive(Debug, Eq, PartialEq)]
pub enum Stmt {
    Expression(ExprId),
    Function(Token, Vec<Token>, Rc<Vec<StmtId>>),
    Declaration(Token, ExprId),
    Block(Vec<StmtId>),
    If(ExprId, StmtId, Option<StmtId>),
    Loop(Option<ExprId>, StmtId, Option<StmtId>),
    Break(Token),
    Continue(Token),
    Return(Token, Option<ExprId>),
}

pub type StmtId = usize;
