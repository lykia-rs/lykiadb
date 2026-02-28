use crate::ast::Span;
use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;
use crate::ast::{Identifier, Literal};
use crate::{Locals, Scopes};
use lykiadb_common::error::InputError;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use super::program::Program;

pub struct Resolver<'a> {
    scopes: Scopes,
    locals: Locals,
    program: &'a Program,
}

impl<'a> Resolver<'a> {
    pub fn resolve(&mut self) -> Result<(Scopes, Locals), ResolveError> {
        self.visit_stmt(&self.program.get_root())?;
        let scopes = self.scopes.clone();
        let locals = self.locals.clone();
        Ok((scopes, locals))
    }

    pub fn new(
        scopes: Scopes,
        program: &'a Program,
        previous_locals: Option<Locals>,
    ) -> Resolver<'a> {
        Resolver {
            scopes,
            locals: previous_locals.unwrap_or_default(),
            program,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn resolve_stmts(&mut self, statements: &Vec<Stmt>) -> Result<(), ResolveError> {
        for statement in statements {
            self.resolve_stmt(statement)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, statement: &Stmt) -> Result<(), ResolveError> {
        self.visit_stmt(statement)?;
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), ResolveError> {
        self.visit_expr(expr)?;
        Ok(())
    }

    fn resolve_local(&mut self, expr_id: usize, name: &Identifier) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name.name) {
                self.locals.insert(expr_id, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn declare(&mut self, name: &Identifier) {
        match self.scopes.last_mut() {
            Some(scope) => scope.insert(name.name.to_string(), false),
            None => return,
        };
    }

    fn define(&mut self, name: &Identifier) {
        match self.scopes.last_mut() {
            Some(scope) => scope.insert(name.name.to_string(), true),
            None => return,
        };
    }
}

impl Resolver<'_> {
    fn visit_expr(&mut self, e: &Expr) -> Result<(), ResolveError> {
        match e {
            Expr::Literal { value, span, .. } => match value {
                Literal::Object(map) => {
                    for item in map.keys() {
                        let value = map.get(item).ok_or(ResolveError::VariableNotFound {
                            span: *span,
                            name: item.to_string(),
                        })?;

                        self.visit_expr(value)?;
                    }
                }
                Literal::Array(items) => {
                    for item in items {
                        self.visit_expr(item)?;
                    }
                }
                _ => (),
            },

            Expr::Grouping { expr, .. } | Expr::Unary { expr, .. } => self.resolve_expr(expr)?,

            Expr::Binary { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            Expr::Variable { name, span, id } => {
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
                self.resolve_local(*id, name);
            }
            Expr::Assignment { dst, expr, id, .. } => {
                self.resolve_expr(expr)?;
                self.resolve_local(*id, dst);
            }
            Expr::Logical { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            Expr::Call { callee, args, .. } => {
                self.resolve_expr(callee)?;

                for argument in args {
                    self.resolve_expr(argument)?;
                }
            }
            Expr::Function {
                name,
                parameters,
                body,
                ..
            } => {
                if let Some(name) = name {
                    self.declare(name);
                    self.define(name);
                }
                self.begin_scope();
                for (ident, _) in parameters {
                    self.declare(ident);
                    self.define(ident);
                }
                self.resolve_stmts(body.as_ref())?;
                self.end_scope();
            }
            Expr::Between {
                lower,
                upper,
                subject,
                ..
            } => {
                self.resolve_expr(lower)?;
                self.resolve_expr(upper)?;
                self.resolve_expr(subject)?;
            }
            Expr::Get { object, .. } => {
                self.resolve_expr(object)?;
            }
            Expr::Set { object, value, .. } => {
                self.resolve_expr(object)?;
                self.resolve_expr(value)?;
            }
            // TODO(vck): SQL should also be resolved:
            Expr::Select { .. }
            | Expr::Insert { .. }
            | Expr::Update { .. }
            | Expr::Delete { .. }
            | Expr::FieldPath { .. } => (),
        };
        Ok(())
    }

    fn visit_stmt(&mut self, s: &Stmt) -> Result<(), ResolveError> {
        match s {
            Stmt::Program { body: stmts, .. } => {
                self.resolve_stmts(stmts)?;
            }
            Stmt::Block { body: stmts, .. } => {
                self.begin_scope();
                self.resolve_stmts(stmts)?;
                self.end_scope();
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => (),
            Stmt::Expression { expr, .. } => {
                self.resolve_expr(expr)?;
            }
            Stmt::Declaration { dst, expr, .. } => {
                self.declare(dst);
                self.resolve_expr(expr)?;
                self.define(dst);
            }
            Stmt::If {
                condition,
                body,
                r#else_body: r#else,
                ..
            } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)?;
                if let Some(r#else) = r#else {
                    self.resolve_stmt(r#else.as_ref())?;
                }
            }
            Stmt::Loop {
                condition,
                body,
                post,
                ..
            } => {
                if let Some(condition) = condition {
                    self.resolve_expr(condition.as_ref())?;
                }
                self.resolve_stmt(body)?;
                if let Some(post) = post {
                    self.resolve_stmt(post.as_ref())?;
                }
            }
            Stmt::Return { expr, .. } => {
                if let Some(expr) = expr {
                    self.resolve_expr(expr.as_ref())?;
                }
            }

            Stmt::Explain { expr, .. } => {
                self.resolve_expr(expr)?;
            }
        };
        Ok(())
    }
}

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum ResolveError {
    #[error("Failed to resolve variable `{name}`")]
    VariableNotFound { span: Span, name: String },
    #[error("{message} at {span:?}")]
    GenericError { span: Span, message: String },
}

impl From<ResolveError> for InputError {
    fn from(value: ResolveError) -> Self {
        let (hint, sp) = match &value {
            ResolveError::GenericError { span, .. } => {
                ("Check variable declarations and scope usage", *span)
            }
            ResolveError::VariableNotFound { span, name } => (
                &format!("Variable `{name}` not found in the current scope") as &str,
                *span,
            ),
        };

        InputError::new(&value.to_string(), hint, Some(sp.into()))
    }
}
