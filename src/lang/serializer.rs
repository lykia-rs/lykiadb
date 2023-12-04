use serde_json::{json, Value};

use super::{
    ast::{
        expr::{Expr, ExprId},
        stmt::{Stmt, StmtId},
        Visitor,
    },
    parser::Parsed,
};
use std::rc::Rc;

impl Parsed {
    pub fn serialize(&mut self) -> String {
        serde_json::to_string_pretty(&json!({
            "type": "Program",
            "body": self.statements.clone().iter().map(|stmt| self.visit_stmt(*stmt).unwrap()).collect::<Vec<_>>()
        })).unwrap()
    }
}

impl Visitor<Value, ()> for Parsed {
    fn visit_expr(&mut self, eidx: ExprId) -> Result<Value, ()> {
        // TODO: Remove clone here
        let a = Rc::clone(&self.arena);
        let e = a.get_expression(eidx);

        let matched: Value = match e {
            Expr::Select(val) => json!({
                "type": "Expr::Select",
                // TODO(vck): Implement rest of the select
            }),
            Expr::Literal(val) => {
                json!({
                    "type": "Expr::Literal",
                    "value": format!("{:?}", val),
                })
            }
            Expr::Grouping(expr) => {
                json!({
                    "type": "Expr::Grouping",
                    "value": self.visit_expr(*expr)?,
                })
            }
            Expr::Unary { token, expr } => {
                json!({
                    "type": "Expr::Unary",
                    "operator": token.span.lexeme.as_ref(),
                    "value": self.visit_expr(*expr)?,
                })
            }
            Expr::Binary { token, left, right } => {
                json!({
                    "type": "Expr::Binary",
                    "left": self.visit_expr(*left)?,
                    "operator": token.span.lexeme.as_ref(),
                    "right": self.visit_expr(*right)?,
                })
            }
            Expr::Variable(tok) => {
                json!({
                    "type": "Expr::Variable",
                    "value": tok.span.lexeme.as_ref(),
                })
            }
            Expr::Assignment { var_tok, expr } => {
                json!({
                    "type": "Expr::Assignment",
                    "variable": var_tok.span.lexeme.as_ref(),
                    "value": self.visit_expr(*expr)?,
                })
            }
            Expr::Logical { left, token, right } => {
                json!({
                    "type": "Expr::Logical",
                    "left": self.visit_expr(*left)?,
                    "operator": token.span.lexeme.as_ref(),
                    "right": self.visit_expr(*right)?,
                })
            }
            Expr::Call {
                callee,
                paren: _,
                args,
            } => {
                json!({
                    "type": "Expr::Call",
                    "callee": self.visit_expr(*callee)?,
                    "args": args.iter().map(|arg| self.visit_expr(*arg).unwrap()).collect::<Vec<_>>(),
                })
            }
            Expr::Function {
                name,
                parameters,
                body,
            } => {
                let fn_name = if name.is_some() {
                    name.as_ref().unwrap().span.lexeme.as_ref()
                } else {
                    "<anonymous>"
                };
                json!({
                    "type": "Expr::Function",
                    "name": fn_name,
                    "parameters": parameters.iter().map(|param| param.span.lexeme.as_ref()).collect::<Vec<_>>(),
                    "body": body.iter().map(|stmt| self.visit_stmt(*stmt).unwrap()).collect::<Vec<_>>(),
                })
            }
        };

        Ok(matched)
    }

    fn visit_stmt(&mut self, sidx: StmtId) -> Result<Value, ()> {
        // TODO: Remove clone here
        let a = Rc::clone(&self.arena);
        let s = a.get_statement(sidx);
        let matched: Value = match s {
            Stmt::Expression(expr) => {
                json!({
                    "type": "Stmt::Expression",
                    "value": self.visit_expr(*expr)?,
                })
            }
            Stmt::Declaration { token, expr } => {
                json!({
                    "type": "Stmt::Declaration",
                    "variable": token.span.lexeme.as_ref(),
                    "value": self.visit_expr(*expr)?,
                })
            }
            Stmt::Block(statements) => {
                json!({
                    "type": "Stmt::Block",
                    "body": statements.iter().map(|stmt| self.visit_stmt(*stmt).unwrap()).collect::<Vec<_>>(),
                })
            }
            Stmt::If {
                condition,
                body,
                r#else,
            } => {
                json!({
                    "type": "Stmt::If",
                    "condition": self.visit_expr(*condition)?,
                    "body": self.visit_stmt(*body)?,
                    "else": if r#else.is_some() {
                        self.visit_stmt(r#else.unwrap())?
                    } else {
                        json!("None")
                    },
                })
            }
            Stmt::Loop {
                condition,
                body,
                post,
            } => {
                json!({
                    "type": "Stmt::Loop",
                    "condition": if condition.is_some() {
                        self.visit_expr(condition.unwrap())?
                    } else {
                        json!("None")
                    },
                    "body": self.visit_stmt(*body)?,
                    "post": if post.is_some() {
                        self.visit_stmt(post.unwrap())?
                    } else {
                        json!("None")
                    },
                })
            }
            Stmt::Break(_) => json!({
                "type": "Stmt::Break",
            }),
            Stmt::Continue(_) => json!({
                "type": "Stmt::Continue",
            }),
            Stmt::Return { token: _, expr } => {
                json!({
                    "type": "Stmt::Return",
                    "value": if expr.is_some() {
                        self.visit_expr(expr.unwrap())?
                    } else {
                        json!("None")
                    },
                })
            }
        };
        Ok(matched)
    }
}
