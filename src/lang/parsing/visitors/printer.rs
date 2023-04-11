use crate::lang::parsing::expr::{Expr, Visitor};
use crate::lang::parsing::token::LiteralValue::{Num, Str, Bool, Nil};

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
            Expr::Literal(v) => format!("{:?}", v),
            Expr::Grouping(expr) => format!("({})", self.visit_expr(expr)),
            Expr::Unary(tok, expr) => format!("{}{}", tok.lexeme.as_ref().unwrap_or(&"".to_string()), self.visit_expr(expr)),
        }
    }
}