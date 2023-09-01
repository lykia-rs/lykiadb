use std::process::exit;
use std::rc::Rc;
use crate::{kw, sym};
use crate::lang::execution::environment::{Environment, Shared};
use crate::lang::parsing::ast::{Expr, Stmt, Visitor};
use crate::lang::execution::primitives::{Function, HaltReason, runtime_err};
use crate::lang::parsing::token::TokenType;
use crate::lang::parsing::token::Keyword::*;
use crate::lang::parsing::token::Symbol::*;
use crate::lang::parsing::token::TokenType::Symbol;
use crate::lang::parsing::token::RV;
use crate::lang::parsing::token::RV::*;
use crate::lang::parsing::token::Token;

macro_rules! bool2num {
    ($val: expr) => {
        if $val { 1.0 } else { 0.0 }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum LoopState {
    Go,
    Broken,
    Continue,
}

#[derive(Debug)]
pub struct Context {
    ongoing_loops: Option<Vec<LoopState>>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            ongoing_loops: None,
        }
    }

    pub fn push_loop(&mut self, state: LoopState) {
        if self.ongoing_loops.is_none() {
            self.ongoing_loops = Some(vec![]);
        }
        self.ongoing_loops.as_mut().unwrap().push(state);
    }

    pub fn pop_loop(&mut self) {
        self.ongoing_loops.as_mut().unwrap().pop();
    }

    pub fn is_loops_empty(&self) -> bool {
        if self.ongoing_loops.is_none() {
            return true;
        }
        return self.ongoing_loops.as_ref().unwrap().is_empty()
    }

    pub fn get_last_loop(&self) -> Option<&LoopState> {
        if self.ongoing_loops.is_none() {
            return None;
        }
        return self.ongoing_loops.as_ref().unwrap().last();
    }

    pub fn set_last_loop(&mut self, to: LoopState) {
        if self.ongoing_loops.is_none() {
            return;
        }
        self.pop_loop();
        self.push_loop(to);
    }
}

pub struct Interpreter {
    env: Shared<Environment>,
    call_stack: Vec<Context>,
}

fn is_value_truthy(rv: RV) -> bool {
    match rv {
        RV::Num(value) => !value.is_nan() && value.abs() > 0.0,
        RV::Str(value) => !value.is_empty(),
        RV::Bool(value) => value,
        RV::Null |
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

    fn eval_unary(&mut self, tok: &Token, expr: &Box<Expr>) -> RV {
        if tok.tok_type == sym!(Minus) {
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

    fn eval_binary(&mut self, left: &Box<Expr>, right: &Box<Expr>, tok: &Token) -> RV {
        let left_eval = self.visit_expr(left);
        let right_eval = self.visit_expr(right);
        let tok_type = tok.tok_type.clone();

        let (left_coerced, right_coerced) = match (&left_eval, &tok_type, &right_eval) {
            (RV::Num(n), _, RV::Bool(bool)) => (RV::Num(*n), RV::Num(bool2num!(*bool))),
            (RV::Bool(bool), _, RV::Num(n)) => (RV::Num(bool2num!(*bool)), RV::Num(*n)),
            (RV::Bool(l), Symbol(Plus), RV::Bool(r)) |
            (RV::Bool(l), Symbol(Minus), RV::Bool(r)) |
            (RV::Bool(l), Symbol(Star), RV::Bool(r)) |
            (RV::Bool(l), Symbol(Slash), RV::Bool(r)) => (RV::Num(bool2num!(*l)), RV::Num(bool2num!(*r))),
            (_, _, _) => (left_eval, right_eval)
        };

        match (left_coerced, tok_type, right_coerced) {
            (RV::Null, Symbol(EqualEqual), RV::Null) => RV::Bool(true),
            (RV::Null, Symbol(BangEqual), RV::Null) => RV::Bool(false),
            //
            (_, Symbol(EqualEqual), RV::Null) |
            (RV::Null, Symbol(EqualEqual), _) => RV::Bool(false),
            //
            (RV::NaN, Symbol(Plus), _) | (_, Symbol(Plus), RV::NaN) |
            (RV::NaN, Symbol(Minus), _) | (_, Symbol(Minus), RV::NaN) |
            (RV::NaN, Symbol(Star), _) | (_, Symbol(Star), RV::NaN) |
            (RV::NaN, Symbol(Slash), _) | (_, Symbol(Slash), RV::NaN) => RV::NaN,
            //
            (RV::Num(l), Symbol(Plus), RV::Num(r)) => RV::Num(l + r),
            (RV::Num(l), Symbol(Minus), RV::Num(r)) => RV::Num(l - r),
            (RV::Num(l), Symbol(Star), RV::Num(r)) => RV::Num(l * r),
            (RV::Num(l), Symbol(Slash), RV::Num(r)) => RV::Num(l / r),
            (RV::Num(l), Symbol(Less), RV::Num(r)) => RV::Bool(l < r),
            (RV::Num(l), Symbol(LessEqual), RV::Num(r)) => RV::Bool(l <= r),
            (RV::Num(l), Symbol(Greater), RV::Num(r)) => RV::Bool(l > r),
            (RV::Num(l), Symbol(GreaterEqual), RV::Num(r)) => RV::Bool(l >= r),
            (RV::Num(l), Symbol(BangEqual), RV::Num(r)) => RV::Bool(l != r),
            (RV::Num(l), Symbol(EqualEqual), RV::Num(r)) => RV::Bool(l == r),
            //
            (RV::Str(l), Symbol(Plus), RV::Str(r)) => RV::Str(Rc::new(l.to_string() + &r.to_string())),
            (RV::Str(l), Symbol(Less), RV::Str(r)) => RV::Bool(l < r),
            (RV::Str(l), Symbol(LessEqual), RV::Str(r)) => RV::Bool(l <= r),
            (RV::Str(l), Symbol(Greater), RV::Str(r)) => RV::Bool(l > r),
            (RV::Str(l), Symbol(GreaterEqual), RV::Str(r)) => RV::Bool(l >= r),
            (RV::Str(l), Symbol(BangEqual), RV::Str(r)) => RV::Bool(l != r),
            (RV::Str(l), Symbol(EqualEqual), RV::Str(r)) => RV::Bool(l == r),
            //
            (RV::Bool(l), Symbol(Less), RV::Bool(r)) => RV::Bool(!l & r),
            (RV::Bool(l), Symbol(LessEqual), RV::Bool(r)) => RV::Bool(l <= r),
            (RV::Bool(l), Symbol(Greater), RV::Bool(r)) => RV::Bool(l & !r),
            (RV::Bool(l), Symbol(GreaterEqual), RV::Bool(r)) => RV::Bool(l >= r),
            (RV::Bool(l), Symbol(BangEqual), RV::Bool(r)) => RV::Bool(l != r),
            (RV::Bool(l), Symbol(EqualEqual), RV::Bool(r)) => RV::Bool(l == r),
            //
            (RV::Str(s), Symbol(Plus), RV::Num(num)) => RV::Str(Rc::new(s.to_string() + &num.to_string())),
            (RV::Num(num), Symbol(Plus), RV::Str(s)) => RV::Str(Rc::new(num.to_string() + &s.to_string())),
            //
            (RV::Str(s), Symbol(Plus), RV::Bool(bool))  => RV::Str(Rc::new(s.to_string() + &bool.to_string())),
            (RV::Bool(bool), Symbol(Plus), RV::Str(s)) => RV::Str(Rc::new(bool.to_string() + &s.to_string())),
            //
            (_, Symbol(Less), _) |
            (_, Symbol(LessEqual), _) |
            (_, Symbol(Greater), _) |
            (_, Symbol(GreaterEqual), _) |
            (_, Symbol(EqualEqual), _) |
            (_, Symbol(BangEqual), _) => RV::Bool(false),
            //
            (_, Symbol(Plus), _) |
            (_, Symbol(Minus), _) |
            (_, Symbol(Star), _) |
            (_, Symbol(Slash), _) => RV::NaN,
            //
            (_, _, _) => RV::Undefined
        }
    }
}

impl Interpreter {
    fn is_loop_at(&self, state: LoopState) -> bool {
        let last_loop = *self.call_stack[0].get_last_loop().unwrap();
        last_loop == state
    }

    fn set_loop_state(&mut self, to: LoopState, from: Option<LoopState>) -> bool {
        if from.is_none() {
            return if !self.call_stack[0].is_loops_empty() {
                self.call_stack[0].set_last_loop(to);
                true
            } else {
                false
            }
        }
        else if self.is_loop_at(from.unwrap()) {
            self.call_stack[0].set_last_loop(to);
        }
        true
    }

    pub fn user_fn_call(&mut self, statements: &Vec<Stmt>, environment: Shared<Environment>) -> Result<RV, HaltReason> {
        self.execute_block(statements, Some(environment))
    }

    pub fn execute_block(&mut self, statements: &Vec<Stmt>, env_opt: Option<Shared<Environment>>) -> Result<RV, HaltReason> {
        let mut env_tmp: Option<Shared<Environment>> = None;

        if let Some(env_opt_unwrapped) = env_opt {
            env_tmp = Some(self.env.clone());
            self.env = env_opt_unwrapped;
        }
        else {
            self.env = Environment::new(Some(self.env.clone()));
        }
        let mut ret = Ok(RV::Undefined);

        for statement in statements {
            ret = self.visit_stmt(statement);
            if ret.is_err() {
                break;
            }
        }
        if let Some(env_tmp_unwrapped) = env_tmp {
            self.env = env_tmp_unwrapped;
        }
        else {
            self.env = self.env.clone().borrow_mut().pop();
        }
        ret
    }
}

impl Visitor<RV, HaltReason> for Interpreter {

    fn visit_expr(&mut self, e: &Expr) -> RV {
        match e {
            Expr::Literal(_, value) => value.clone(),
            Expr::Grouping(_, expr) => self.visit_expr(expr),
            Expr::Unary(_, tok, expr) => self.eval_unary(tok, expr),
            Expr::Binary(_, tok, left, right) => self.eval_binary(left, right, tok),
            Expr::Variable(_, tok) => self.env.borrow_mut().read(tok.lexeme.as_ref().unwrap()).unwrap(),
            Expr::Assignment(_, tok, expr) => {
                let evaluated = self.visit_expr(expr);
                if let Err(HaltReason::Error(msg)) = self.env.borrow_mut().assign(tok.lexeme.as_ref().unwrap().to_string(), evaluated.clone()) {
                    runtime_err(&msg, tok.line);
                    exit(1);
                }
                evaluated
            },
            Expr::Logical(_, left, tok, right) => {
                let is_true = is_value_truthy(self.visit_expr(left));

                if (tok.tok_type == kw!(Or) && is_true) || (tok.tok_type == kw!(And) && !is_true) {
                    return RV::Bool(is_true);
                }

                RV::Bool(is_value_truthy(self.visit_expr(right)))
            },
            Expr::Call(_, callee, paren, arguments) => {
                let eval = self.visit_expr(callee);

                if let Callable(arity, callable) = eval {
                    if arity.is_some() && arity.unwrap() != arguments.len() {
                        runtime_err(&format!("Function expects {} arguments, while provided {}.", arity.unwrap(), arguments.len()), paren.line);
                        exit(1);
                    }
                    let args_evaluated: Vec<RV> = arguments.iter().map(|arg| self.visit_expr(arg)).collect();
                    self.call_stack.insert(0, Context::new());
                    let val = callable.call(self, args_evaluated.as_slice());
                    self.call_stack.remove(0);
                    match val {
                        Err(HaltReason::Error(msg))=> panic!("{}", msg),
                        Err(HaltReason::Return(unpacked_val)) => {
                            unpacked_val
                        }
                        Ok(unpacked_val) => {
                            unpacked_val
                        }
                    }
                }
                else {
                    runtime_err("Expression does not yield a callable", paren.line);
                    exit(1);
                }
            }
        }
    }

    fn visit_stmt(&mut self, e: &Stmt) -> Result<RV, HaltReason> {
        if !self.call_stack[0].is_loops_empty() && *self.call_stack[0].get_last_loop().unwrap() != LoopState::Go {
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
                        return Err(runtime_err("Variable name cannot be empty", tok.line));
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
                self.call_stack[0].push_loop(LoopState::Go);
                while !self.is_loop_at(LoopState::Broken) && (condition.is_none() || is_value_truthy(self.visit_expr(condition.as_ref().unwrap()))) {
                    self.visit_stmt(stmt)?;
                    self.set_loop_state(LoopState::Go, Some(LoopState::Continue));
                    if let Some(post) = post_body {
                        self.visit_stmt(post)?;
                    }
                }
                self.call_stack[0].pop_loop();
            },
            Stmt::Break(token) => {
                if !self.set_loop_state(LoopState::Broken, None) {
                    return Err(runtime_err("Unexpected break statement", token.line));
                }
            },
            Stmt::Continue(token) => {
                if !self.set_loop_state(LoopState::Continue, None) {
                    return Err(runtime_err("Unexpected continue statement", token.line));
                }
            },
            Stmt::Return(_token, expr) => {
                if expr.is_some() {
                    let ret = self.visit_expr(expr.as_ref().unwrap());
                    return Err(HaltReason::Return(ret));
                }
                return Err(HaltReason::Return(RV::Undefined));
            },
            Stmt::Function(token, parameters, body) => {
                let name = token.lexeme.as_ref().unwrap().to_string();
                let fun = Function::UserDefined(
                    name.clone(),
                    Rc::clone(body),
                    parameters.into_iter().map(|x| x.lexeme.as_ref().unwrap().to_string()).collect(),
                    self.env.clone(),
                );

                self.env.borrow_mut().declare(name, Callable(Some(parameters.len()), Rc::new(fun)));
            }
        }
        Ok(RV::Undefined)
    }
}