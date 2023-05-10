use std::rc::Rc;
use crate::lang::parsing::token::{LiteralValue, Token};
pub type BExpr = Box<Expr>;
pub type BStmt = Box<Stmt>;

pub trait Visitor<T> {
    fn visit_expr(&mut self, e: &Expr) -> T;
    fn visit_stmt(&mut self, e: &Stmt) -> T;
}

#[derive(Debug)]
pub enum Expr {
    Binary(Token, BExpr, BExpr),
    Grouping(BExpr),
    Literal(LiteralValue),
    Unary(Token, BExpr),
    Variable(Token),
    Assignment(Token, BExpr),
    Logical(BExpr, Token, BExpr),
    Call(BExpr, Token, Vec<BExpr>),
}

pub enum Stmt {
    Expression(BExpr),
    Function(Token, Vec<Token>, Rc<Vec<Stmt>>),
    Declaration(Token, BExpr),
    Block(Vec<Stmt>),
    If(BExpr, BStmt, Option<BStmt>),
    Loop(Option<BExpr>, BStmt, Option<BStmt>),
    Break(Token),
    Continue(Token)
}
