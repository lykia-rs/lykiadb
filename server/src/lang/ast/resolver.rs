use crate::lang::ast::expr::{Expr, ExprId};
use crate::lang::ast::stmt::{Stmt, StmtId};
use crate::lang::ast::visitor::VisitorMut;
use crate::lang::tokens::token::Span;
use crate::lang::{Identifier, Literal};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use super::program::AstArena;

pub struct Resolver<'a> {
    scopes: Vec<FxHashMap<String, bool>>,
    locals: FxHashMap<usize, usize>,
    arena: &'a AstArena,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolveError {
    GenericError { span: Span, message: String },
}

impl<'a> Resolver<'a> {
    pub fn resolve(
        &mut self,
        arg @ (payload, root): ((), StmtId),
    ) -> Result<(Vec<FxHashMap<String, bool>>, FxHashMap<usize, usize>), ResolveError> {
        self.visit_stmt(arg)?;
        let scopes = self.scopes.clone();
        let locals = self.locals.clone();
        Ok((scopes, locals))
    }

    pub fn new(scopes: Vec<FxHashMap<String, bool>>, arena: &'a AstArena) -> Resolver<'a> {
        Resolver {
            scopes,
            locals: FxHashMap::default(),
            arena,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn resolve_stmts(&mut self, statements: &Vec<StmtId>) {
        for statement in statements {
            self.resolve_stmt(*statement);
        }
    }

    fn resolve_stmt(&mut self, statement: StmtId) {
        self.visit_stmt(((), statement)).unwrap();
    }

    fn resolve_expr(&mut self, expr: ExprId) {
        self.visit_expr(((), expr)).unwrap();
    }

    fn resolve_local(&mut self, expr: ExprId, name: &Identifier) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name.name) {
                self.locals.insert(expr.0, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn declare(&mut self, name: &Identifier) {
        if self.scopes.is_empty() {
            return;
        }
        let last = self.scopes.last_mut();
        last.unwrap().insert(name.name.to_string(), false);
    }

    fn define(&mut self, name: &Identifier) {
        if self.scopes.is_empty() {
            return;
        }
        let last = self.scopes.last_mut();
        last.unwrap().insert(name.name.to_string(), true);
    }
}

impl<'a> VisitorMut<(), ResolveError> for Resolver<'a> {
    fn visit_expr(&mut self, (_, eidx): ((), ExprId)) -> Result<(), ResolveError> {
        let a = self.arena;
        let e = a.get_expression(eidx);
        match e {
            Expr::Literal {
                raw: _,
                span: _,
                value,
            } => match value {
                Literal::Object(map) => {
                    for item in map.keys() {
                        self.visit_expr(((), *map.get(item).unwrap()))?;
                    }
                }
                Literal::Array(items) => {
                    for item in items {
                        self.visit_expr(((), *item))?;
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
            Expr::Variable { name, span } => {
                if !self.scopes.is_empty() {
                    let last_scope = self.scopes.last().unwrap();
                    let value = last_scope.get(&name.name);
                    if value.is_some() && !(*value.unwrap()) {
                        return Err(ResolveError::GenericError {
                            span: *span,
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
                    self.declare(&name.as_ref().unwrap().clone());
                    self.define(&name.as_ref().unwrap().clone());
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
            | Expr::Insert {
                command: _,
                span: _,
            }
            | Expr::Update {
                command: _,
                span: _,
            }
            | Expr::Delete {
                command: _,
                span: _,
            } => (),
        };
        Ok(())
    }

    fn visit_stmt(&mut self, (_, sidx): ((), StmtId)) -> Result<(), ResolveError> {
        let a = self.arena;
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
        };
        Ok(())
    }
}
