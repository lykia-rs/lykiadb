use std::rc::Rc;
use rustc_hash::FxHashMap;
use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{HaltReason, RV};
use crate::lang::parsing::ast::{Expr, Stmt, Visitor};

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
}

impl Visitor<RV, HaltReason> for Resolver {

    fn visit_expr(&mut self, e: &Expr) -> RV {
        match e {
            Expr::Literal(_) => ,
            Expr::Grouping(expr) => ,
            Expr::Unary(tok, expr) => ,
            Expr::Binary(tok, left, right) => ,
            Expr::Variable(tok) => ,
            Expr::Assignment(tok, expr) => {

            },
            Expr::Logical(left, tok, right) => {

            },
            Expr::Call(callee, paren, arguments) => {

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

            }
        }
        Ok(RV::Undefined)
    }
}