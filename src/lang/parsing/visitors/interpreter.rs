use crate::lang::parsing::expr::{Expr, Visitor};
use crate::lang::parsing::token::LiteralValue::{Num, Str, Bool, Nil};

pub struct Interpreter;
impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter
    }
}

impl Visitor<String> for Interpreter {
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