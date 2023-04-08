use crate::lang::parsing::token::{LiteralValue, Token};
use crate::lang::parsing::token::LiteralValue::{Bool, Num, Str, Nil};

pub trait Visitor<T> {
    fn visit_expr(&mut self, e: &Expr) -> T;
}

#[derive(Debug)]
pub enum Expr {
    Binary(Token, Box<Expr>, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(LiteralValue),
    Unary(Token, Box<Expr>),
}

pub struct Printer;
impl Printer {
    pub fn new() -> Printer {
        Printer
    }
}

impl Visitor<String> for Printer {
    fn visit_expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Binary(tok, left, right)
                => format!("({} {} {})",self.visit_expr(left), tok.lexeme.as_ref().unwrap_or(&"".to_string()), self.visit_expr(right)),
            Expr::Literal(Str(value)) => format!("'{}'", value),
            Expr::Literal(Num(value)) => format!("{}", value),
            Expr::Literal(Bool(value)) => format!("_{}_", value),
            Expr::Literal(Nil) => "_nil_".to_string(),
            Expr::Grouping(expr) => format!("({})", self.visit_expr(expr)),
            Expr::Unary(tok, expr) => format!("{}{}", tok.lexeme.as_ref().unwrap_or(&"".to_string()), self.visit_expr(expr)),
        }
    }
}
