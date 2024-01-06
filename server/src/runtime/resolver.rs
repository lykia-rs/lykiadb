use crate::lang::ast::expr::{Expr, ExprId};
use crate::lang::ast::stmt::{Stmt, StmtId};
use crate::lang::ast::{Literal, ParserArena, VisitorMut};
use crate::lang::token::Token;
use crate::runtime::types::RV;
use rustc_hash::FxHashMap;
use std::rc::Rc;

pub struct Resolver {
    scopes: Vec<FxHashMap<String, bool>>,
    locals: FxHashMap<ExprId, usize>,
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
        self.locals.get(&eid).copied()
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
            if self.scopes[i].contains_key(name.literal.as_ref().unwrap().as_str().unwrap()) {
                self.locals.insert(expr, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    pub fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let last = self.scopes.last_mut();
        last.unwrap().insert(
            name.literal.as_ref().unwrap().as_str().unwrap().to_string(),
            false,
        );
    }

    pub fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let last = self.scopes.last_mut();
        last.unwrap().insert(
            name.literal.as_ref().unwrap().as_str().unwrap().to_string(),
            true,
        );
    }
}

impl VisitorMut<RV, ResolveError> for Resolver {
    fn visit_expr(&mut self, eidx: ExprId) -> Result<RV, ResolveError> {
        let a = Rc::clone(&self.arena);
        let e = a.get_expression(eidx);
        match e {
            Expr::Literal {
                raw: _,
                span: _,
                value,
            } => match value {
                Literal::Object(map) => {
                    for item in map.keys() {
                        self.visit_expr(*map.get(item).unwrap())?;
                    }
                }
                Literal::Array(items) => {
                    for item in items {
                        self.visit_expr(*item)?;
                    }
                }
                _ => (),
            },
            Expr::Grouping { expr, span: _ } => self.resolve_expr(*expr),
            Expr::Unary {
                operation: _,
                expr,
                span: _,
            } => self.resolve_expr(*expr),
            Expr::Binary {
                operation: _,
                left,
                right,
                span: _,
            } => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            Expr::Variable { name, span: _ } => {
                if !self.scopes.is_empty() {
                    let last_scope = self.scopes.last().unwrap();
                    let value = last_scope.get(name.literal.as_ref().unwrap().as_str().unwrap());
                    if value.is_some() && !(*value.unwrap()) {
                        return Err(ResolveError::GenericError {
                            token: name.clone(),
                            message: "Can't read local variable in its own initializer."
                                .to_string(),
                        });
                    }
                }
                self.resolve_local(eidx, name);
            }
            Expr::Assignment { dst, expr, span: _ } => {
                self.resolve_expr(*expr);
                self.resolve_local(eidx, dst);
            }
            Expr::Logical {
                left,
                operation: _,
                right,
                span: _,
            } => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            Expr::Call {
                callee,
                args,
                span: _,
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
                span: _,
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
            Expr::Get {
                object,
                name: _,
                span: _,
            } => {
                self.resolve_expr(*object);
            }
            Expr::Set {
                object,
                name: _,
                value,
                span: _,
            } => {
                self.resolve_expr(*object);
                self.resolve_expr(*value);
            }
            Expr::Select { query: _, span: _ }
            | Expr::Insert { command: _, span: _ }
            | Expr::Update { command: _, span: _ }
            | Expr::Delete { command: _, span: _ }
             => (),
        };
        Ok(RV::Undefined)
    }

    fn visit_stmt(&mut self, sidx: StmtId) -> Result<RV, ResolveError> {
        let a = Rc::clone(&self.arena);
        let s = a.get_statement(sidx);
        match s {
            Stmt::Program {
                body: stmts,
                span: _,
            } => {
                self.resolve_stmts(stmts);
            }
            Stmt::Block {
                body: stmts,
                span: _,
            } => {
                self.begin_scope();
                self.resolve_stmts(stmts);
                self.end_scope();
            }
            Stmt::Break { span: _ } | Stmt::Continue { span: _ } => (),
            Stmt::Expression { expr, span: _ } => {
                self.resolve_expr(*expr);
            }
            Stmt::Declaration { dst, expr, span: _ } => {
                self.declare(dst);
                self.resolve_expr(*expr);
                self.define(dst);
            }
            Stmt::If {
                condition,
                body,
                r#else_body: r#else,
                span: _,
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
                span: _,
            } => {
                if condition.is_some() {
                    self.resolve_expr(*condition.as_ref().unwrap());
                }
                self.resolve_stmt(*body);
                if post.is_some() {
                    self.resolve_stmt(*post.as_ref().unwrap());
                }
            }
            Stmt::Return { span: _, expr } => {
                if expr.is_some() {
                    self.resolve_expr(expr.unwrap());
                }
            }
        }
        Ok(RV::Undefined)
    }
}
