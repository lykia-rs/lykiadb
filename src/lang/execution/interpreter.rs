use std::process::exit;
use std::rc::Rc;
use crate::lang::execution::environment::{Environment, Shared};
use crate::lang::parsing::ast::{BExpr, Expr, Stmt, Visitor};
use crate::lang::parsing::token::LiteralValue::{Bool, Nil, Num, Str};
use crate::lang::parsing::token::Token;
use crate::lang::parsing::token::TokenType::*;
use crate::lang::execution::error::runtime_err;
use crate::lang::execution::primitives::{Function, Reason, RV};
use crate::lang::execution::primitives::RV::Callable;

macro_rules! bool2num {
    ($val: expr) => {
        if $val { 1.0 } else { 0.0 }
    }
}

#[derive(PartialEq, Debug)]
pub enum LoopState {
    Go,
    Broken,
    Continue,
}

#[derive(Debug)]
pub struct Context {
    ongoing_loops: Vec<LoopState>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            ongoing_loops: vec![],
        }
    }
}

pub struct Interpreter {
    env: Shared<Environment>,
    call_stack: Vec<Context>
}

fn is_value_truthy(rv: RV) -> bool {
    match rv {
        RV::Num(value) => !value.is_nan() && value.abs() > 0.0,
        RV::Str(value) => !value.is_empty(),
        RV::Bool(value) => value,
        RV::Nil |
        RV::Undefined |
        RV::NaN => false,
        _ => true
    }
}

impl Interpreter {
    pub fn new(env: Shared<Environment>) -> Interpreter {
        Interpreter {
            env,
            call_stack: vec![Context::new()]
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

impl Interpreter {
    fn is_loop_at(&self, state: LoopState) -> bool {
        *self.call_stack[0].ongoing_loops.last().unwrap() == state
    }

    fn set_loop_state(&mut self, to: LoopState, from: Option<LoopState>) -> bool {
        if from.is_none() {
            return if !self.call_stack[0].ongoing_loops.is_empty() {
                let last_item = self.call_stack[0].ongoing_loops.last_mut();
                *last_item.unwrap() = to;
                true
            } else {
                false
            }
        }
        else if self.is_loop_at(from.unwrap()) {
            let last_item = self.call_stack[0].ongoing_loops.last_mut();
            *last_item.unwrap() = to;
        }
        true
    }

    pub fn user_fn_call(&mut self, statements: &Vec<Stmt>, pairs_opt: Option<Vec<(String, RV)>>) -> Result<RV, Reason> {
        self.execute_block(statements, pairs_opt)
    }

    pub fn execute_block(&mut self, statements: &Vec<Stmt>, pairs_opt: Option<Vec<(String, RV)>>) -> Result<RV, Reason> {
        self.env = Environment::new(Some(self.env.clone()));
        if let Some(pairs) = pairs_opt {
            for pair in pairs {
                self.env.borrow_mut().declare(pair.0, pair.1)
            }
        }
        let mut ret = Ok(RV::Undefined);
        for statement in statements {
            ret = self.visit_stmt(statement);
            if ret.is_err() {
                break;
            }
        }
        self.env = self.env.clone().borrow_mut().pop();
        ret
    }
}

impl Visitor<RV, Reason> for Interpreter {

    fn visit_expr(&mut self, e: &Expr) -> RV {
        match e {
            Expr::Literal(Str(value)) => RV::Str(value.clone()),
            Expr::Literal(Num(value)) => RV::Num(*value),
            Expr::Literal(Bool(value)) => RV::Bool(*value),
            Expr::Literal(Nil) => RV::Nil,
            Expr::Grouping(expr) => self.visit_expr(expr),
            Expr::Unary(tok, expr) => self.eval_unary(tok, expr),
            Expr::Binary(tok, left, right) => self.eval_binary(left, right, tok),
            Expr::Variable(tok) => self.env.borrow_mut().read(tok.lexeme.as_ref().unwrap()).unwrap(),
            Expr::Assignment(tok, expr) => {
                let evaluated = self.visit_expr(expr);
                if let Err(Reason::Error(msg)) = self.env.borrow_mut().assign(tok.lexeme.as_ref().unwrap().to_string(), evaluated.clone()) {
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
            Expr::Call(callee, paren, arguments) => {
                let eval = self.visit_expr(callee);

                if let Callable(callable) = eval {
                    let arity = callable.arity();
                    if arity.is_some() && arity.unwrap() != arguments.len() {
                        runtime_err(&format!("Function expects {} arguments, while provided {}.", arity.unwrap(), arguments.len()), paren.line);
                        exit(1);
                    }
                    let args_evaluated: Vec<RV> = arguments.iter().map(|arg| self.visit_expr(arg)).collect();
                    self.call_stack.insert(0, Context::new());
                    let val = callable.call(self, args_evaluated);
                    self.call_stack.remove(0);
                    match val {
                        Err(Reason::Return(unpacked_val)) => {
                            unpacked_val
                        }
                        Err(Reason::Error(msg))=> panic!("{}", msg),
                        _ => RV::Undefined
                    }
                }
                else {
                    runtime_err("Expression does not yield a callable", paren.line);
                    exit(1);
                }
            }
        }
    }

    fn visit_stmt(&mut self, e: &Stmt) -> Result<RV, Reason> {
        if !self.call_stack[0].ongoing_loops.is_empty() && *self.call_stack[0].ongoing_loops.last().unwrap() == LoopState::Continue {
            return Ok(RV::Undefined);
        }
        match e {
            Stmt::Expression(expr) => {
                return Ok(self.visit_expr(expr));
            },
            Stmt::Declaration(tok, expr) => {
                match &tok.lexeme {
                    Some(var_name) => {
                        let evaluated = self.visit_expr(expr);
                        self.env.borrow_mut().declare(var_name.to_string(), evaluated);
                    },
                    None => {
                        runtime_err("Variable name cannot be empty", tok.line);
                    }
                }
            },
            Stmt::Block(statements) => { return Ok(self.execute_block(statements, None))?; },
            Stmt::If(condition, if_stmt, else_optional) => {
                if is_value_truthy(self.visit_expr(condition)) {
                    self.visit_stmt(if_stmt)?;
                }
                else if let Some(else_stmt) = else_optional {
                    self.visit_stmt(else_stmt)?;
                }
            },
            Stmt::Loop(condition, stmt, post_body) => {
                self.call_stack[0].ongoing_loops.push(LoopState::Go);
                while !self.is_loop_at(LoopState::Broken) && (condition.is_none() || is_value_truthy(self.visit_expr(condition.as_ref().unwrap()))) {
                    self.visit_stmt(stmt)?;
                    self.set_loop_state(LoopState::Go, Some(LoopState::Continue));
                    if let Some(post) = post_body {
                        self.visit_stmt(post)?;
                    }
                }
                self.call_stack[0].ongoing_loops.pop();
            },
            Stmt::Break(token) => {
                if !self.set_loop_state(LoopState::Broken, None) {
                    runtime_err("Unexpected break statement", token.line);
                }
            },
            Stmt::Continue(token) => {
                if !self.set_loop_state(LoopState::Continue, None) {
                    runtime_err("Unexpected continue statement", token.line);
                }
            },
            Stmt::Return(_token, expr) => {
                if expr.is_some() {
                    let ret = self.visit_expr(expr.as_ref().unwrap());
                    return Err(Reason::Return(ret));
                }
                return Err(Reason::Return(RV::Undefined));
            },
            Stmt::Function(token, parameters, body) => {
                let fun = Function {
                    parameters: (*parameters).iter().map(|x| x.lexeme.as_ref().unwrap().clone()).collect(),
                    body: body.clone()
                };

                self.env.borrow_mut().declare(token.lexeme.as_ref().unwrap().to_string(), Callable(Rc::new(fun)));
            }
        }
        Ok(RV::Undefined)
    }
}