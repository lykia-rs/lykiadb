use serde_json::{json, Value};

use super::{
    ast::{
        expr::{Expr, ExprId},
        stmt::{Stmt, StmtId},
        ImmutableVisitor,
    },
    parser::Program,
};
use std::rc::Rc;

impl Program {
    pub fn to_json(&self) -> Value {
        json!(self.visit_stmt(self.root).unwrap())
    }
    pub fn serialize(&self) -> String {
        serde_json::to_string_pretty(&self.to_json()).unwrap()
    }
}

impl ToString for Program {
    fn to_string(&self) -> String {
        self.serialize().clone()
    }
}

impl ImmutableVisitor<Value, ()> for Program {
    fn visit_expr(&self, eidx: ExprId) -> Result<Value, ()> {
        // TODO: Remove clone here
        let a = Rc::clone(&self.arena);
        let e = a.get_expression(eidx);

        let matched: Value = match e {
            Expr::Select { span: _, query: _ } => json!({
                "type": "Expr::Select",
                // TODO(vck): Implement rest of the select
            }),
            Expr::Literal {
                raw,
                span: _,
                value,
            } => {
                json!({
                    "type": "Expr::Literal",
                    "value": format!("{:?}", value),
                    "raw": raw,
                })
            }
            Expr::Grouping { expr, span: _ } => {
                json!({
                    "type": "Expr::Grouping",
                    "value": self.visit_expr(*expr)?,
                })
            }
            Expr::Unary {
                symbol,
                expr,
                span: _,
            } => {
                json!({
                    "type": "Expr::Unary",
                    "operator": symbol,
                    "value": self.visit_expr(*expr)?,
                })
            }
            Expr::Binary {
                symbol,
                left,
                right,
                span: _,
            } => {
                json!({
                    "type": "Expr::Binary",
                    "left": self.visit_expr(*left)?,
                    "operator": symbol,
                    "right": self.visit_expr(*right)?,
                })
            }
            Expr::Variable { name, span: _ } => {
                json!({
                    "type": "Expr::Variable",
                    "value": name.lexeme.as_ref(),
                })
            }
            Expr::Assignment { dst, expr, span: _ } => {
                json!({
                    "type": "Expr::Assignment",
                    "variable": dst.lexeme.as_ref(),
                    "value": self.visit_expr(*expr)?,
                })
            }
            Expr::Logical {
                left,
                symbol,
                right,
                span: _,
            } => {
                json!({
                    "type": "Expr::Logical",
                    "left": self.visit_expr(*left)?,
                    "operator": symbol,
                    "right": self.visit_expr(*right)?,
                })
            }
            Expr::Call {
                callee,
                span: _,
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
                span: _,
            } => {
                let fn_name = if name.is_some() {
                    name.as_ref().unwrap().lexeme.as_ref().unwrap().to_string()
                } else {
                    "<anonymous>".to_string()
                };
                json!({
                    "type": "Expr::Function",
                    "name": fn_name,
                    "parameters": parameters.iter().map(|param| param.lexeme.as_ref()).collect::<Vec<_>>(),
                    "body": body.iter().map(|stmt| self.visit_stmt(*stmt).unwrap()).collect::<Vec<_>>(),
                })
            }
            Expr::Get {
                object,
                name,
                span: _,
            } => {
                json!({
                    "type": "Expr::Get",
                    "object": self.visit_expr(*object)?,
                    "property": name.lexeme.as_ref(),
                })
            }
            Expr::Set {
                object,
                name,
                value,
                span: _,
            } => {
                json!({
                    "type": "Expr::Set",
                    "object": self.visit_expr(*object)?,
                    "property": name.lexeme.as_ref(),
                    "value": self.visit_expr(*value)?,
                })
            }
        };

        Ok(matched)
    }

    fn visit_stmt(&self, sidx: StmtId) -> Result<Value, ()> {
        // TODO: Remove clone here
        let a = Rc::clone(&self.arena);
        let s = a.get_statement(sidx);
        let matched: Value = match s {
            Stmt::Program { stmts, span: _ } => {
                json!({
                    "type": "Stmt::Program",
                    "body": stmts.iter().map(|stmt| self.visit_stmt(*stmt).unwrap()).collect::<Vec<_>>(),
                })
            }
            Stmt::Block { stmts, span: _ } => {
                json!({
                    "type": "Stmt::Block",
                    "body": stmts.iter().map(|stmt| self.visit_stmt(*stmt).unwrap()).collect::<Vec<_>>(),
                })
            }
            Stmt::Expression { expr, span: _ } => {
                json!({
                    "type": "Stmt::Expression",
                    "value": self.visit_expr(*expr)?,
                })
            }
            Stmt::Declaration { dst, expr, span: _ } => {
                json!({
                    "type": "Stmt::Declaration",
                    "variable": dst.lexeme.as_ref().unwrap(),
                    "value": self.visit_expr(*expr)?,
                })
            }
            Stmt::If {
                condition,
                body,
                r#else,
                span: _,
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
                span: _,
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
            Stmt::Break { span: _ } => json!({
                "type": "Stmt::Break",
            }),
            Stmt::Continue { span: _ } => json!({
                "type": "Stmt::Continue",
            }),
            Stmt::Return { expr, span: _ } => {
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
