use crate::lang::ast::expr::{Expr, ExprId};
use crate::lang::ast::stmt::{Stmt, StmtId};
use crate::lang::ast::{ParserArena, Visitor};
use crate::lang::token::Token;
use crate::runtime::types::RV;
use rustc_hash::FxHashMap;
use std::rc::Rc;

pub struct Resolver {
    scopes: Vec<FxHashMap<String, bool>>,
    locals: FxHashMap<usize, usize>,
    arena: Rc<ParserArena>,
}

#[derive(Debug, Clone)]
pub enum ResolveError {
    GenericError { token: Token, message: String },
}

impl Resolver {
    pub fn new(arena: Rc<ParserArena>) -> Resolver {
        Resolver {
            scopes: vec![],
            locals: FxHashMap::default(),
            arena,
        }
    }

    pub fn get_distance(&self, eid: ExprId) -> Option<usize> {
        self.locals.get(&eid.0).copied()
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn resolve_stmts(&mut self, statements: &Vec<StmtId>) {
        for statement in statements {
            self.resolve_stmt(*statement);
        }
    }

    pub fn resolve_stmt(&mut self, statement: StmtId) {
        self.visit_stmt(statement).unwrap();
    }

    pub fn resolve_expr(&mut self, expr: ExprId) {
        self.visit_expr(expr).unwrap();
    }

    pub fn resolve_local(&mut self, expr: ExprId, name: &Token) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name.span.lexeme.as_ref().to_string()) {
                self.locals.insert(expr.0, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    pub fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let last = self.scopes.last_mut();
        last.unwrap()
            .insert(name.span.lexeme.as_ref().to_string(), false);
    }

    pub fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let last = self.scopes.last_mut();
        last.unwrap()
            .insert(name.span.lexeme.as_ref().to_string(), true);
    }
}

impl Visitor<RV, ResolveError> for Resolver {
    fn visit_expr(&mut self, eidx: ExprId) -> Result<RV, ResolveError> {
        let a = Rc::clone(&self.arena);
        let e = a.get_expression(eidx);
        match e {
            Expr::Literal(_) => (),
            Expr::Grouping(expr) => self.resolve_expr(*expr),
            Expr::Unary { token: _, expr } => self.resolve_expr(*expr),
            Expr::Binary {
                token: _,
                left,
                right,
            } => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            Expr::Variable(tok) => {
                if !self.scopes.is_empty() {
                    let last_scope = self.scopes.last().unwrap();
                    let value = last_scope.get(&tok.span.lexeme.to_string());
                    if value.is_some() && !(*value.unwrap()) {
                        return Err(ResolveError::GenericError {
                            token: tok.clone(),
                            message: "Can't read local variable in its own initializer."
                                .to_string(),
                        });
                    }
                }
                self.resolve_local(eidx, tok);
            }
            Expr::Assignment { var_tok, expr } => {
                self.resolve_expr(*expr);
                self.resolve_local(eidx, var_tok);
            }
            Expr::Logical {
                left,
                token: _,
                right,
            } => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            Expr::Call {
                callee,
                paren: _,
                args,
            } => {
                self.resolve_expr(*callee);

                for argument in args {
                    self.resolve_expr(*argument);
                }
            }
            Expr::Function {
                name,
                parameters,
                body,
            } => {
                if name.is_some() {
                    self.declare(name.as_ref().unwrap());
                    self.define(name.as_ref().unwrap());
                }
                self.begin_scope();
                for param in parameters {
                    self.declare(param);
                    self.define(param);
                }
                self.resolve_stmts(body.as_ref());
                self.end_scope();
            }
            Expr::Select(_) => (),
        };
        Ok(RV::Undefined)
    }

    fn visit_stmt(&mut self, sidx: StmtId) -> Result<RV, ResolveError> {
        let a = Rc::clone(&self.arena);
        let s = a.get_statement(sidx);
        match s {
            Stmt::Break(_token) | Stmt::Continue(_token) => (),
            Stmt::Expression(expr) => {
                self.resolve_expr(*expr);
            }
            Stmt::Declaration { token, expr } => {
                self.declare(token);
                self.resolve_expr(*expr);
                self.define(token);
            }
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve_stmts(statements);
                self.end_scope();
            }
            Stmt::If {
                condition,
                body,
                r#else,
            } => {
                self.resolve_expr(*condition);
                self.resolve_stmt(*body);
                if r#else.is_some() {
                    self.resolve_stmt(*r#else.as_ref().unwrap());
                }
            }
            Stmt::Loop {
                condition,
                body,
                post,
            } => {
                self.resolve_expr(*condition.as_ref().unwrap());
                self.resolve_stmt(*body);
                if post.is_some() {
                    self.resolve_stmt(*post.as_ref().unwrap());
                }
            }
            Stmt::Return { token: _, expr } => {
                if expr.is_some() {
                    self.resolve_expr(expr.unwrap());
                }
            }
        }
        Ok(RV::Undefined)
    }
}
