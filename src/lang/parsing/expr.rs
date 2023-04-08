use crate::lang::parsing::token::{LiteralValue, Token};
use crate::lang::parsing::token::LiteralValue::{Bool, Num, Str, Nil};

pub trait Visitor<T> {
    fn visit_expr(&mut self, e: &Expr) -> T;
}

#[derive(Debug)]
pub enum Expr {
    BinaryExpr(Token, Box<Expr>, Box<Expr>),
    GroupingExpr(Box<Expr>),
    LiteralExpr(LiteralValue),
    UnaryExpr(Token, Box<Expr>),
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
            Expr::BinaryExpr(tok, left, right)
                => format!("({} {} {})",self.visit_expr(left), tok.lexeme.as_ref().unwrap_or(&"".to_string()), self.visit_expr(right)),
            Expr::LiteralExpr(Str(value)) => format!("'{}'", value),
            Expr::LiteralExpr(Num(value)) => format!("{}", value),
            Expr::LiteralExpr(Bool(value)) => format!("_{}_", value),
            Expr::LiteralExpr(Nil) => "_nil_".to_string(),
            Expr::GroupingExpr(expr) => format!("({})", self.visit_expr(expr)),
            Expr::UnaryExpr(tok, expr) => format!("{}{}", tok.lexeme.as_ref().unwrap_or(&"".to_string()), self.visit_expr(expr)),
        }
    }
}
