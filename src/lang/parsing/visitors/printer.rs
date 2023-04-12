use crate::lang::parsing::ast::{Ast, Expr, Stmt, Visitor};
use crate::lang::parsing::token::LiteralValue::{Str, Bool, Nil};

pub struct Printer;
impl Printer {
    pub fn new() -> Printer {
        Printer
    }
}

impl Visitor<String> for Printer {
    fn visit(&mut self, a: &Ast) -> Option<String> {
        let mut out_buf = String::new();
        for stmt in a {
            out_buf.push_str(&self.visit_stmt(stmt));
        }
        Some(out_buf)
    }

    fn visit_expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Binary(tok, left, right)
                => format!("({} {} {})",self.visit_expr(left), tok.lexeme.as_ref().unwrap_or(&"".to_string()), self.visit_expr(right)),
            Expr::Literal(v) => format!("{:?}", v),
            Expr::Grouping(expr) => format!("({})", self.visit_expr(expr)),
            Expr::Unary(tok, expr) => format!("{}{}", tok.lexeme.as_ref().unwrap_or(&"".to_string()), self.visit_expr(expr)),
        }
    }
    fn visit_stmt(&mut self, e: &Stmt) -> String {
        match e {
            Stmt::Print(expr) => {
                let val = self.visit_expr(expr);
                format!("PRINT({:?})", val)
            },
            Stmt::Expression(expr) => {
                self.visit_expr(expr)
            }
        }
    }
}