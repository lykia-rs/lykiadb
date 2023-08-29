use std::process::exit;
use std::rc::Rc;
use rustc_hash::FxHashMap;
use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{HaltReason, runtime_err, RV};
use crate::lang::parsing::ast::{BExpr, Expr, Stmt, Visitor};
use crate::lang::parsing::token::Token;

pub struct Resolver {
    interpreter: Rc<Interpreter>,
    scopes: Vec<FxHashMap<String, bool>>
}

impl Resolver {
    pub fn new(interpreter: Rc<Interpreter>) -> Resolver {
        Resolver {
            interpreter,
            scopes: vec![]
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
        // TODO
    }

    pub fn resolve_expr(&mut self, expr: &BExpr) {
        // TODO
    }

    pub fn resolve_local(&mut self, expr: &BExpr, name: &Token) {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(&name.lexeme.unwrap().to_string()) {
                self.interpreter.resolve(expr, scopes.size() - 1 - i);
                return;
            }
        }
    }

    pub fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        self.scopes.last().as_mut().unwrap().insert(name.lexeme.unwrap().to_string(), false);
    }

    pub fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        self.scopes.last().as_mut().unwrap().insert(name.lexeme.unwrap().to_string(), true);
    }
}

impl Visitor<RV, HaltReason> for Resolver {

    fn visit_expr(&mut self, e: &Expr) -> RV {
        match e {
            Expr::Literal(_) => (),
            Expr::Grouping(expr) => self.resolve_expr(expr),
            Expr::Unary(tok, expr) => self.resolve_expr(expr),
            Expr::Binary(tok, left, right) => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Variable(tok) => {
                if !self.scopes.is_empty() &&
                    !self.scopes.last().unwrap().get(name.lexeme.unwrap().to_string()) {
                    runtime_err(&"Can't read local variable in its own initializer.", tok.line);
                    exit(1);
                }

                self.resolve_local(expr, expr.name);
            },
            expr @ Expr::Assignment(name, value) => {
                self.resolve_expr(value);
                self.resolve_local(expr, name);
            },
            Expr::Logical(left, tok, right) => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            },
            Expr::Call(callee, paren, arguments) => {
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
            Stmt::Expression(expr) => {

            },
            Stmt::Declaration(tok, expr) => {

            },
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve_stmts(statements);
                self.end_scope();
            },
            Stmt::If(condition, if_stmt, else_optional) => {

            },
            Stmt::Loop(condition, stmt, post_body) => {

            },
            Stmt::Break(token) => {

            },
            Stmt::Continue(token) => {

            },
            Stmt::Return(_token, expr) => {

            },
            Stmt::Function(token, parameters, body) => {
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