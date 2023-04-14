use crate::lang::parsing::ast::{BExpr, Expr, Stmt, Visitor};
use crate::lang::parsing::token::LiteralValue::{Num, Str, Bool, Nil};
use crate::lang::parsing::token::Token;
use crate::lang::parsing::token::TokenType::*;
use crate::lang::runtime::environment::{Environment, RV};
use crate::lang::runtime::error::runtime_err;

macro_rules! bool2num {
    ($val: expr) => {
        if $val { 1.0 } else { 0.0 }
    }
}

pub struct Interpreter {
    env: Environment
}

impl Interpreter {
    pub fn new(env: Environment) -> Interpreter {
        Interpreter {
            env
        }
    }

    fn eval_unary(&mut self, tok: &Token, expr: &BExpr) -> RV {
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
    }

    fn eval_binary(&mut self, left: &BExpr, right: &BExpr, tok: &Token) -> RV {
        let left_eval = self.visit_expr(left);
        let right_eval = self.visit_expr(right);
        let tok_type = tok.tok_type.clone();

        let (left_coerced, right_coerced) = match (&left_eval, &tok_type, &right_eval) {
            (RV::Num(n), _, RV::Bool(bool)) => (RV::Num(*n), RV::Num(bool2num!(*bool))),
            (RV::Bool(bool), _, RV::Num(n)) => (RV::Num(bool2num!(*bool)), RV::Num(*n)),
            (RV::Bool(l), Plus, RV::Bool(r)) |
            (RV::Bool(l), Minus, RV::Bool(r)) |
            (RV::Bool(l), Star, RV::Bool(r)) |
            (RV::Bool(l), Slash, RV::Bool(r)) => (RV::Num(bool2num!(*l)), RV::Num(bool2num!(*r))),
            (_, _, _) => (left_eval, right_eval)
        };

        match (left_coerced, tok_type, right_coerced) {
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
            (RV::Bool(l), Less, RV::Bool(r)) => RV::Bool(!l & r),
            (RV::Bool(l), LessEqual, RV::Bool(r)) => RV::Bool(l <= r),
            (RV::Bool(l), Greater, RV::Bool(r)) => RV::Bool(l & !r),
            (RV::Bool(l), GreaterEqual, RV::Bool(r)) => RV::Bool(l >= r),
            (RV::Bool(l), BangEqual, RV::Bool(r)) => RV::Bool(l != r),
            (RV::Bool(l), EqualEqual, RV::Bool(r)) => RV::Bool(l == r),
            //
            (RV::Str(str), Plus, RV::Num(num)) => RV::Str(str.to_string() + &num.to_string()),
            (RV::Num(num), Plus, RV::Str(str)) => RV::Str(num.to_string() + &str),
            //
            (RV::Str(s), Plus, RV::Bool(bool))  => RV::Str(s.to_string() + &bool.to_string()),
            (RV::Bool(bool), Plus, RV::Str(s)) => RV::Str(bool.to_string() + &s),
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
}

impl Visitor<RV> for Interpreter {
    fn visit_expr(&mut self, e: &Expr) -> RV {
        match e {
            Expr::Literal(Str(value)) => RV::Str(value.clone()),
            Expr::Literal(Num(value)) => RV::Num(*value),
            Expr::Literal(Bool(value)) => RV::Bool(*value),
            Expr::Literal(Nil) => RV::Nil,
            Expr::Grouping(expr) => self.visit_expr(expr).clone(),
            Expr::Unary(tok, expr) => self.eval_unary(tok, expr).clone(),
            Expr::Binary(tok, left, right) => self.eval_binary(left, right, tok).clone(),
            Expr::Variable(tok) => self.env.read(&tok.lexeme.as_ref().unwrap_or(&"".to_string())).unwrap_or(&RV::Nil).clone()
        }
    }

    fn visit_stmt(&mut self, e: &Stmt) -> RV {
        match e {
            Stmt::Print(expr) => {
                println!("{:?}", self.visit_expr(expr));
                RV::Undefined
            },
            Stmt::Expression(expr) => {
                self.visit_expr(expr)
            }
            Stmt::Declaration(tok, expr) => {
                match &tok.lexeme {
                    Some(var_name) => {
                        let evaluated = self.visit_expr(expr).clone();
                        self.env.declare(var_name.clone(), evaluated);
                    },
                    None => {
                        runtime_err("Variable name cannot be empty", tok.line);
                    }
                }
                RV::Undefined
            }
        }
    }
}