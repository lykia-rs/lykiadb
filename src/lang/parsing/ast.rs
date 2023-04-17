use crate::lang::parsing::token::{LiteralValue, Token};
pub type BExpr = Box<Expr>;
// pub type Ast = Vec<Stmt>;

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
}

pub enum Stmt {
    Expression(BExpr),
    Print(BExpr),
    Declaration(Token, BExpr),
    Block(Vec<Stmt>),
    If(BExpr, Box<Stmt>, Option<Box<Stmt>>),
    While(BExpr, Box<Stmt>)
}
