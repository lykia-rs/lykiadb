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
pub enum RuntimeValue {
    Str(String),
    Num(f32),
    Bool(bool),
    Nil
}

impl Visitor<RuntimeValue> for Interpreter {
    fn visit_expr(&mut self, e: &Expr) -> RuntimeValue {
        match e {
            Expr::Binary(tok, left, right)  => {
                let left_eval = self.visit_expr(left);
                let right_eval = self.visit_expr(right);
                let tok_type = tok.tok_type.clone();

                if let RuntimeValue::Num(left_value) = left_eval {
                    if let RuntimeValue::Num(right_value) = right_eval {
                        return match tok_type {
                            Plus => RuntimeValue::Num(left_value + right_value),
                            Minus => RuntimeValue::Num(left_value - right_value),
                            Star => RuntimeValue::Num(left_value * right_value),
                            Slash => RuntimeValue::Num(left_value / right_value),
                            Less => RuntimeValue::Bool(left_value < right_value),
                            LessEqual => RuntimeValue::Bool(left_value <= right_value),
                            Greater => RuntimeValue::Bool(left_value > right_value),
                            GreaterEqual => RuntimeValue::Bool(left_value >= right_value),
                            BangEqual => RuntimeValue::Bool(left_value != right_value),
                            EqualEqual => RuntimeValue::Bool(left_value == right_value),
                            _ => {
                                runtime_err(&format!("Unexpected operator '{}' in arithmetic operation", &tok.lexeme.clone().unwrap_or(" ".to_string())), tok.line);
                                exit(1);
                            }
                        };
                    }
                }

                if let RuntimeValue::Str(left_value) = left_eval {
                    if let RuntimeValue::Str(right_value) = right_eval {
                        return match tok_type {
                            Plus => RuntimeValue::Str(left_value + &right_value),
                            _ => {
                                runtime_err(&format!("Unexpected operator '{}' in string operation", &tok.lexeme.clone().unwrap_or(" ".to_string())), tok.line);
                                exit(1);
                            }
                        };
                    }
                }

                RuntimeValue::Num(1.0)
            }
            Expr::Literal(Str(value)) => RuntimeValue::Str(value.clone()),
            Expr::Literal(Num(value)) => RuntimeValue::Num(value.clone()),
            Expr::Literal(Bool(value)) => RuntimeValue::Bool(value.clone()),
            Expr::Literal(Nil) => RuntimeValue::Nil,
            Expr::Grouping(expr) => self.visit_expr(expr),
            Expr::Unary(tok, expr) => {
                if tok.tok_type == Minus {
                    let val = self.visit_expr(expr);
                    match val {
                        RuntimeValue::Num(value) => RuntimeValue::Num(-value),
                        _ => {
                            runtime_err(&format!("Non-numeric {:?} cannot be negated", val), tok.line);
                            exit(1);
                        }
                    }
                }
                else {
                    match self.visit_expr(expr) {
                        RuntimeValue::Num(value) => RuntimeValue::Bool(value == 0.0 || value.is_nan()),
                        RuntimeValue::Str(value) => RuntimeValue::Bool(value.is_empty()),
                        RuntimeValue::Bool(value) => RuntimeValue::Bool(!value),
                        RuntimeValue::Nil => RuntimeValue::Bool(true),
                    }
                }
            },
        }
    }
}