use serde_json::{json, Value};

use super::{
    ast::{
        expr::{Expr, ExprId},
        sql::{SqlExpr, SqlSelect},
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
    fn visit_select(&self, select: &SqlSelect) -> Result<Value, ()> {
        let order_by: Option<Value> = select
            .order_by
            .as_ref()
            .map(|x| {
                x.iter()
                    .map(|order| {
                        let expr = if let SqlExpr::Default(eidx) = order.expr {
                            self.visit_expr(eidx).unwrap()
                        } else {
                            panic!("Not implemented");
                        };
                        let val = json!({
                            "expr": expr,
                            "ordering": format!("{:?}", order.ordering),
                        });
                        val
                    })
                    .collect()
            });

        let limit: Option<Value> = select.limit.as_ref().map(|x| {
            let count_part = if let SqlExpr::Default(eidx) = x.count {
                self.visit_expr(eidx).unwrap()
            } else {
                panic!("Not implemented");
            };

            let offset_part = if x.offset.is_some() {
                if let SqlExpr::Default(eidx) = x.offset.as_ref().unwrap() {
                    self.visit_expr(*eidx).unwrap()
                } else {
                    panic!("Not implemented");
                }
            } else {
                json!(serde_json::Value::Null)
            };

            json!({
                "count": count_part,
                "offset": offset_part
            })
        });
        /*
        {
            pub core: SqlSelectCore,
            pub compound: Vec<(SqlCompoundOperator, SqlSelectCore)>,
            pub order_by: Option<Vec<(SqlExpr, SqlOrdering)>>,
            pub limit: Option<(SqlExpr, Option<SqlExpr>)>,
        }
        */
        Ok(json!({
            "order_by": order_by,
            "limit": limit
        }))
    }

    fn visit_expr(&self, eidx: ExprId) -> Result<Value, ()> {
        // TODO: Remove clone here
        let a = Rc::clone(&self.arena);
        let e = a.get_expression(eidx);

        let matched: Value = match e {
            Expr::Select { span: _, query } => json!({
                "type": "Expr::Select",
                "value": self.visit_select(query).unwrap(),
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
                    "expr": self.visit_expr(*expr)?,
                })
            }
            Expr::Unary {
                operator,
                expr,
                span: _,
            } => {
                json!({
                    "type": "Expr::Unary",
                    "operator": operator,
                    "expr": self.visit_expr(*expr)?,
                })
            }
            Expr::Binary {
                operator,
                left,
                right,
                span: _,
            } => {
                json!({
                    "type": "Expr::Binary",
                    "left": self.visit_expr(*left)?,
                    "operator": operator,
                    "right": self.visit_expr(*right)?,
                })
            }
            Expr::Variable { name, span: _ } => {
                json!({
                    "type": "Expr::Variable",
                    "name": name.lexeme.as_ref(),
                })
            }
            Expr::Assignment { dst, expr, span: _ } => {
                json!({
                    "type": "Expr::Assignment",
                    "dst": dst.lexeme.as_ref(),
                    "expr": self.visit_expr(*expr)?,
                })
            }
            Expr::Logical {
                left,
                operator,
                right,
                span: _,
            } => {
                json!({
                    "type": "Expr::Logical",
                    "left": self.visit_expr(*left)?,
                    "operator": operator,
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
                    "name": name.lexeme.as_ref(),
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
                    "name": name.lexeme.as_ref(),
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
            Stmt::Program { body, span: _ } => {
                json!({
                    "type": "Stmt::Program",
                    "body": body.iter().map(|stmt| self.visit_stmt(*stmt).unwrap()).collect::<Vec<_>>(),
                })
            }
            Stmt::Block { body, span: _ } => {
                json!({
                    "type": "Stmt::Block",
                    "body": body.iter().map(|stmt| self.visit_stmt(*stmt).unwrap()).collect::<Vec<_>>(),
                })
            }
            Stmt::Expression { expr, span: _ } => {
                json!({
                    "type": "Stmt::Expression",
                    "expr": self.visit_expr(*expr)?,
                })
            }
            Stmt::Declaration { dst, expr, span: _ } => {
                json!({
                    "type": "Stmt::Declaration",
                    "variable": dst.lexeme.as_ref().unwrap(),
                    "expr": self.visit_expr(*expr)?,
                })
            }
            Stmt::If {
                condition,
                body,
                r#else_body,
                span: _,
            } => {
                json!({
                    "type": "Stmt::If",
                    "condition": self.visit_expr(*condition)?,
                    "body": self.visit_stmt(*body)?,
                    "else_body": if r#else_body.is_some() {
                        self.visit_stmt(r#else_body.unwrap())?
                    } else {
                        json!(serde_json::Value::Null)
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
                        json!(serde_json::Value::Null)
                    },
                    "body": self.visit_stmt(*body)?,
                    "post": if post.is_some() {
                        self.visit_stmt(post.unwrap())?
                    } else {
                        json!(serde_json::Value::Null)
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
                    "expr": if expr.is_some() {
                        self.visit_expr(expr.unwrap())?
                    } else {
                        json!(serde_json::Value::Null)
                    },
                })
            }
        };
        Ok(matched)
    }
}
