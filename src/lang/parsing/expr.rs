use crate::lang::parsing::token::{LiteralValue, Token};
pub type Ast = Box<Expr>;

pub trait Visitor<T> {
    fn visit_expr(&mut self, e: &Expr) -> T;
}

#[derive(Debug)]
pub enum Expr {
    Binary(Token, Ast, Ast),
    Grouping(Ast),
    Literal(LiteralValue),
    Unary(Token, Ast),
}

