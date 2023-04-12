use crate::lang::parsing::expr::{Expr, Visitor};
use crate::lang::parsing::token::LiteralValue::{Num, Str, Bool, Nil};
use crate::lang::parsing::token::{Token, TokenType};
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

macro_rules! bool2num {
    ($val: ident) => {
        if $val { 1.0 } else { 0.0 }
    }
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
                    (RV::Bool(l), Plus, RV::Bool(r)) => RV::Num(bool2num!(l) + bool2num!(r)),
                    (RV::Bool(l), Minus, RV::Bool(r)) => RV::Num(bool2num!(l) - bool2num!(r)),
                    (RV::Bool(l), Star, RV::Bool(r)) => RV::Num(bool2num!(l) * bool2num!(r)),
                    (RV::Bool(l), Slash, RV::Bool(r)) => RV::Num(bool2num!(l) / bool2num!(r)),
                    (RV::Bool(l), Less, RV::Bool(r)) => RV::Bool(!l & r),
                    (RV::Bool(l), LessEqual, RV::Bool(r)) => RV::Bool(l <= r),
                    (RV::Bool(l), Greater, RV::Bool(r)) => RV::Bool(l & !r),
                    (RV::Bool(l), GreaterEqual, RV::Bool(r)) => RV::Bool(l >= r),
                    (RV::Bool(l), BangEqual, RV::Bool(r)) => RV::Bool(l != r),
                    (RV::Bool(l), EqualEqual, RV::Bool(r)) => RV::Bool(l == r),
                    //
                    (RV::Str(str), Plus, RV::Num(num)) => RV::Str(str + &num.to_string()),
                    (RV::Num(num), Plus, RV::Str(str)) => RV::Str(num.to_string() + &str),
                    //
                    (RV::Str(s), Plus, RV::Bool(bool))  => RV::Str(s + &bool.to_string()),
                    (RV::Bool(bool), Plus, RV::Str(s)) => RV::Str(bool.to_string() + &s),
                    //
                    (RV::Num(n), Plus, RV::Bool(bool)) |
                    (RV::Bool(bool), Plus, RV::Num(n)) => RV::Num(n + bool2num!(bool)),
                    (RV::Num(n), Minus, RV::Bool(bool)) => RV::Num(n - bool2num!(bool)),
                    (RV::Bool(bool), Minus, RV::Num(n)) => RV::Num(bool2num!(bool) - n),
                    (RV::Num(n), Star, RV::Bool(bool)) |
                    (RV::Bool(bool), Star, RV::Num(n)) => RV::Num(n * bool2num!(bool)),
                    (RV::Num(n), Slash, RV::Bool(bool)) => RV::Num(n / bool2num!(bool)),
                    (RV::Bool(bool), Slash, RV::Num(n)) => RV::Num(bool2num!(bool) / n),
                    (RV::Bool(bool), Less, RV::Num(n)) => RV::Bool(bool2num!(bool) < n),
                    (RV::Bool(bool), LessEqual, RV::Num(n)) => RV::Bool(bool2num!(bool) <= n),
                    (RV::Bool(bool), Greater, RV::Num(n)) => RV::Bool(bool2num!(bool) > n),
                    (RV::Bool(bool), GreaterEqual, RV::Num(n)) => RV::Bool(bool2num!(bool) >= n),
                    (RV::Num(n), Less, RV::Bool(bool)) => RV::Bool(n < bool2num!(bool)),
                    (RV::Num(n), LessEqual, RV::Bool(bool)) => RV::Bool(n <= bool2num!(bool)),
                    (RV::Num(n), Greater, RV::Bool(bool)) => RV::Bool(n > bool2num!(bool)),
                    (RV::Num(n), GreaterEqual, RV::Bool(bool)) => RV::Bool(n >= bool2num!(bool)),
                    (RV::Bool(bool), BangEqual, RV::Num(n)) |
                    (RV::Num(n), BangEqual, RV::Bool(bool)) => RV::Bool(bool2num!(bool) != n),
                    (RV::Bool(bool), EqualEqual, RV::Num(n)) |
                    (RV::Num(n), EqualEqual, RV::Bool(bool))  => RV::Bool(bool2num!(bool) == n),
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
                    (_, BangEqual, _) => RV::Bool(false),
                    //
                    (_, Plus, _) |
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