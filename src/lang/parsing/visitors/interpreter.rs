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
    Undefined,
    NaN,
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
                    (RV::Str(l), Less, RV::Str(r)) => RV::Bool(l < r),
                    (RV::Str(l), LessEqual, RV::Str(r)) => RV::Bool(l <= r),
                    (RV::Str(l), Greater, RV::Str(r)) => RV::Bool(l > r),
                    (RV::Str(l), GreaterEqual, RV::Str(r)) => RV::Bool(l >= r),
                    (RV::Str(l), BangEqual, RV::Str(r)) => RV::Bool(l != r),
                    (RV::Str(l), EqualEqual, RV::Str(r)) => RV::Bool(l == r),
                    //
                    (RV::Str(str), Plus, RV::Num(num)) => RV::Str(str + &num.to_string()),
                    (RV::Num(num), Plus, RV::Str(str)) => RV::Str(num.to_string() + &str),
                    //
                    (RV::Str(str), Plus, RV::Bool(bool)) |
                    (RV::Bool(bool), Plus, RV::Str(str)) => RV::Str(str + if bool { "true" } else {"false"}),
                    //
                    (RV::Bool(l), BangEqual, RV::Bool(r)) => RV::Bool(l != r),
                    (RV::Bool(l), EqualEqual, RV::Bool(r)) => RV::Bool(l == r),
                    //
                    (RV::Nil, EqualEqual, RV::Nil) => RV::Bool(true),
                    (RV::Nil, BangEqual, RV::Nil) => RV::Bool(false),
                    //
                    (_, EqualEqual, RV::Nil) |
                    (RV::Nil, EqualEqual, _) => RV::Bool(false),
                    //
                    (RV::NaN, Plus, _) | (_, Plus, RV::NaN) |
                    (RV::NaN, Minus, _) | (_, Minus, RV::NaN) |
                    (RV::NaN, Star, _) | (_, Star, RV::NaN) |
                    (RV::NaN, Slash, _) | (_, Slash, RV::NaN) => RV::NaN,
                    //
                    (_, Less, _) |
                    (_, LessEqual, _) |
                    (_, Greater, _) |
                    (_, GreaterEqual, _) |
                    (_, EqualEqual, _) |
                    (_, BangEqual, _) => RV::Bool(true),
                    //
                    (_, Minus, _) |
                    (_, Star, _) |
                    (_, Slash, _) => RV::NaN,
                    //
                    (_, _, _) => RV::Undefined
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
                        RV::Bool(true) => RV::Num(-1.0),
                        RV::Bool(false) => RV::Num(0.0),
                        _ => RV::NaN,
                    }
                }
                else {
                    match self.visit_expr(expr) {
                        RV::Num(value) => RV::Bool(value == 0.0 || value.is_nan()),
                        RV::Str(value) => RV::Bool(value.is_empty()),
                        RV::Bool(value) => RV::Bool(!value),
                        RV::Nil |
                        RV::Undefined |
                        RV::NaN
                        => RV::Bool(true),
                    }
                }
            },
        }
    }
}