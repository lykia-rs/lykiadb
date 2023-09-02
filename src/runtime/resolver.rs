use std::process::exit;
use rustc_hash::FxHashMap;
use crate::runtime::interpreter::{HaltReason, runtime_err};
use crate::lang::ast::{Expr, Stmt, Visitor};
use crate::lang::token::Token;
use crate::lang::types::RV;
use uuid::Uuid;

pub struct Resolver {
    scopes: Vec<FxHashMap<String, bool>>,
    locals: FxHashMap<Uuid, usize>,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            scopes: vec![],
            locals: FxHashMap::default()
        }
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn resolve_stmts(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    pub fn resolve_stmt(&mut self, statement: &Stmt) {
        self.visit_stmt(statement);
    }

    pub fn resolve_expr(&mut self, expr: &Box<Expr>) {
        self.visit_expr(expr);
    }

    pub fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme.as_ref().unwrap().to_string()) {
                self.locals.insert(expr.id(), self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    pub fn declare(&mut self, _name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        // self.scopes.last().as_mut().unwrap().insert(name.lexeme.unwrap().to_string(), false);
    }

    pub fn define(&mut self, _name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        /*let mut last = self.scopes.last().as_mut().unwrap();
        last.insert(name.lexeme.as_mut().unwrap().to_string(), true);
        */
    }
}

impl Visitor<RV, HaltReason> for Resolver {

    fn visit_expr(&mut self, e: &Expr) -> RV {
        match e {
            Expr::Literal(_, _) => (),
            Expr::Grouping(_, expr) => self.resolve_expr(expr),
            Expr::Unary(_, _tok, expr) => self.resolve_expr(expr),
            Expr::Binary(_, _tok, left, right) => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            expr @ Expr::Variable(_, tok) => {
                if !self.scopes.is_empty() &&
                    !*(self.scopes.last().unwrap().get(&tok.lexeme.as_ref().unwrap().to_string()).unwrap()) {
                    runtime_err(&"Can't read local variable in its own initializer.", tok.line);
                    exit(1);
                }

                self.resolve_local(expr, tok);
            },
            expr @ Expr::Assignment(_, name, value) => {
                self.resolve_expr(value);
                self.resolve_local(expr, name);
            },
            Expr::Logical(_, left, _tok, right) => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            },
            Expr::Call(_, callee, _paren, arguments) => {
                self.resolve_expr(callee);

                for argument in arguments {
                    self.resolve_expr(argument);
                }
            }
        };
        RV::Undefined
    }

    fn visit_stmt(&mut self, e: &Stmt) -> Result<RV, HaltReason> {

        match e {
            Stmt::Break(_token) |
            Stmt::Continue(_token) => (),
            Stmt::Expression(expr) => {
                self.resolve_expr(expr);
            },
            Stmt::Declaration(_tok, expr) => {
                self.resolve_expr(expr);
            },
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve_stmts(statements);
                self.end_scope();
            },
            Stmt::If(condition, if_stmt, else_optional) => {
                self.resolve_expr(condition);
                self.resolve_stmt(if_stmt);
                self.resolve_stmt(else_optional.as_ref().unwrap());
            },
            Stmt::Loop(condition, stmt, post_body) => {
                self.resolve_expr(condition.as_ref().unwrap());
                self.resolve_stmt(stmt);
                self.resolve_stmt(post_body.as_ref().unwrap());
            },
            Stmt::Return(_token, expr) => {
                if expr.is_some() {
                    self.resolve_expr(&expr.as_ref().unwrap());
                }
            },
            Stmt::Function(_token, parameters, body) => {
                self.begin_scope();
                for param in parameters {
                    self.declare(param);
                    self.define(param);
                }
                self.resolve_stmts(body.as_ref());
                self.end_scope();
            }
        }
        Ok(RV::Undefined)
    }
}