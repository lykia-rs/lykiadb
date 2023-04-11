use std::process::exit;
use crate::lang::parsing::error::runtime_err;
use crate::lang::parsing::expr::{Expr, Visitor};
use crate::lang::parsing::token::LiteralValue::{Num, Str, Bool, Nil};
use crate::lang::parsing::token::TokenType::*;

pub struct Interpreter;
impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter
    }
}

#[derive(Debug)]
pub enum RV {
    Str(String),
    Num(f32),
    Bool(bool),
    Nil
}

impl Visitor<RV> for Interpreter {
    fn visit_expr(&mut self, e: &Expr) -> RV {
        match e {
            Expr::Binary(tok, left, right)  => {
                let left_eval = self.visit_expr(left);
                let right_eval = self.visit_expr(right);
                let tok_type = tok.tok_type.clone();

                match (left_eval, tok_type, right_eval) {
                    (RV::Num(l), Plus, RV::Num(r)) => RV::Num(l + r),
                    (RV::Num(l), Minus, RV::Num(r)) => RV::Num(l - r),
                    (RV::Num(l), Star, RV::Num(r)) => RV::Num(l * r),
                    (RV::Num(l), Slash, RV::Num(r)) => RV::Num(l / r),
                    (RV::Num(l), Less, RV::Num(r)) => RV::Bool(l < r),
                    (RV::Num(l), LessEqual, RV::Num(r)) => RV::Bool(l <= r),
                    (RV::Num(l), Greater, RV::Num(r)) => RV::Bool(l > r),
                    (RV::Num(l), GreaterEqual, RV::Num(r)) => RV::Bool(l >= r),
                    (RV::Num(l), BangEqual, RV::Num(r)) => RV::Bool(l != r),
                    (RV::Num(l), EqualEqual, RV::Num(r)) => RV::Bool(l == r),
                    //
                    (RV::Str(l), Plus, RV::Str(r)) => RV::Str(l + &r),
                    (RV::Str(l), Plus, RV::Num(r)) => RV::Str(l + &r.to_string()),
                    (RV::Num(l), Plus, RV::Str(r)) => RV::Str(l.to_string() + &r),
                    //
                    (_, _, _) => panic!("Invalid operation")
                }
            }
            Expr::Literal(Str(value)) => RV::Str(value.clone()),
            Expr::Literal(Num(value)) => RV::Num(value.clone()),
            Expr::Literal(Bool(value)) => RV::Bool(value.clone()),
            Expr::Literal(Nil) => RV::Nil,
            Expr::Grouping(expr) => self.visit_expr(expr),
            Expr::Unary(tok, expr) => {
                if tok.tok_type == Minus {
                    let val = self.visit_expr(expr);
                    match val {
                        RV::Num(value) => RV::Num(-value),
                        _ => {
                            runtime_err(&format!("Non-numeric {:?} cannot be negated", val), tok.line);
                            exit(1);
                        }
                    }
                }
                else {
                    match self.visit_expr(expr) {
                        RV::Num(value) => RV::Bool(value == 0.0 || value.is_nan()),
                        RV::Str(value) => RV::Bool(value.is_empty()),
                        RV::Bool(value) => RV::Bool(!value),
                        RV::Nil => RV::Bool(true),
                    }
                }
            },
        }
    }
}