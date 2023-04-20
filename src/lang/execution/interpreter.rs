use std::rc::Rc;
use std::time;
use crate::lang::parsing::ast::{BExpr, Expr, Stmt, Visitor};
use crate::lang::parsing::token::LiteralValue::{Num, Str, Bool, Nil};
use crate::lang::parsing::token::Token;
use crate::lang::parsing::token::TokenType::*;
use crate::lang::execution::environment::{EnvironmentStack, RV};
use crate::lang::execution::error::runtime_err;

macro_rules! bool2num {
    ($val: expr) => {
        if $val { 1.0 } else { 0.0 }
    }
}

#[derive(PartialEq)]
pub enum LoopHandle {
    Go,
    Broken,
    Continue
}

pub struct Interpreter {
    env: EnvironmentStack,
    ongoing_loops: Vec<LoopHandle>
}

fn is_value_truthy(rv: RV) -> bool {
    match rv {
        RV::Num(value) => !value.is_nan() && value.abs() > 0.0,
        RV::Str(value) => !value.is_empty(),
        RV::Bool(value) => value,
        RV::Nil |
        RV::Undefined |
        RV::NaN => false
    }
}

impl Interpreter {
    pub fn new(env: EnvironmentStack) -> Interpreter {
        Interpreter {
            env,
            ongoing_loops: vec![]
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
            RV::Bool(is_value_truthy(self.visit_expr(expr)))
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
            (RV::Str(l), Plus, RV::Str(r)) => RV::Str(Rc::new(l.to_string() + &r.to_string())),
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
            (RV::Str(s), Plus, RV::Num(num)) => RV::Str(Rc::new(s.to_string() + &num.to_string())),
            (RV::Num(num), Plus, RV::Str(s)) => RV::Str(Rc::new(num.to_string() + &s.to_string())),
            //
            (RV::Str(s), Plus, RV::Bool(bool))  => RV::Str(Rc::new(s.to_string() + &bool.to_string())),
            (RV::Bool(bool), Plus, RV::Str(s)) => RV::Str(Rc::new(bool.to_string() + &s.to_string())),
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
            Expr::Grouping(expr) => self.visit_expr(expr),
            Expr::Unary(tok, expr) => self.eval_unary(tok, expr),
            Expr::Binary(tok, left, right) => self.eval_binary(left, right, tok),
            Expr::Variable(tok) => self.env.read(tok.lexeme.as_ref().unwrap()).unwrap().clone(),
            Expr::Assignment(tok, expr) => {
                let evaluated = self.visit_expr(expr);
                if let Err(msg) = self.env.assign(tok.lexeme.as_ref().unwrap().to_string(), evaluated.clone()) {
                    runtime_err(&msg, tok.line)
                }
                evaluated
            },
            Expr::Logical(left, tok, right) => {
                let is_true = is_value_truthy(self.visit_expr(left));

                if (tok.tok_type == Or && is_true) || (tok.tok_type == And && !is_true) {
                    return RV::Bool(is_true);
                }

                RV::Bool(is_value_truthy(self.visit_expr(right)))
            },
            Expr::Clock() => {
                if let Ok(n) = time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
                    RV::Num(n.as_secs_f64())
                }
                else {
                    RV::Num(0.0)
                }
            }
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
            },
            Stmt::Declaration(tok, expr) => {
                match &tok.lexeme {
                    Some(var_name) => {
                        let evaluated = self.visit_expr(expr);
                        self.env.declare(var_name.to_string(), evaluated);
                    },
                    None => {
                        runtime_err("Variable name cannot be empty", tok.line);
                    }
                }
                RV::Undefined
            },
            Stmt::Block(statements) => {
                self.env.push();
                for statement in statements {
                    self.visit_stmt(statement);
                }
                self.env.pop();
                RV::Undefined
            },
            Stmt::If(condition, if_stmt, else_optional) => {
                if is_value_truthy(self.visit_expr(condition)) {
                    self.visit_stmt(if_stmt);
                }
                else if let Some(else_stmt) = else_optional {
                    self.visit_stmt(else_stmt);
                }
                RV::Undefined
            },
            Stmt::Break(token) => {
                if self.ongoing_loops.len() > 0 {
                    let last_item = self.ongoing_loops.last_mut();
                    *last_item.unwrap() = LoopHandle::Broken;
                    return RV::Undefined;
                }
                runtime_err("Unexpected break statement", token.line);
                RV::Undefined
            }
            Stmt::While(condition, stmt) => {
                self.ongoing_loops.push(LoopHandle::Go);
                while *self.ongoing_loops.last().unwrap() == LoopHandle::Go && is_value_truthy(self.visit_expr(condition)) {
                    self.visit_stmt(stmt);
                }
                self.ongoing_loops.pop();
                RV::Undefined
            },
            Stmt::For(definition, bool_condition, increment, inner_stmt) => {
                self.ongoing_loops.push(LoopHandle::Go);
                self.visit_stmt(definition);
                while *self.ongoing_loops.last().unwrap() == LoopHandle::Go && is_value_truthy(self.visit_expr(bool_condition)) {
                    self.visit_stmt(inner_stmt);
                    self.visit_expr(increment);
                }
                self.ongoing_loops.pop();
                RV::Undefined
            }
        }
    }
}