use serde_json::{json, Value};

use super::{
    ast::{
        expr::{Expr, ExprId},
        sql::{SqlCollectionSubquery, SqlExpr, SqlProjection, SqlSelect, SqlSelectCore},
        stmt::{Stmt, StmtId},
        ImmutableVisitor,
    },
    parser::Program,
};
use std::rc::Rc;

pub struct ProgramSerializer<'a> {
    pub program: &'a Program,
}

impl<'a> ProgramSerializer<'a> {
    pub fn new(program: &'a Program) -> ProgramSerializer<'a> {
        ProgramSerializer { program }
    }
    pub fn to_json(&self) -> Value {
        json!(self.visit_stmt(self.program.root).unwrap())
    }
    pub fn serialize(&self) -> String {
        serde_json::to_string_pretty(&self.to_json()).unwrap()
    }
}

impl<'a> ToString for ProgramSerializer<'a> {
    fn to_string(&self) -> String {
        self.serialize().clone()
    }
}

impl<'a> ProgramSerializer<'a> {
    fn visit_sql_expr(&self, sql_expr: &SqlExpr) -> Value {
        if let SqlExpr::Default(eidx) = sql_expr {
            self.visit_expr(*eidx).unwrap()
        } else {
            panic!("Not implemented");
        }
    }

    fn visit_sql_select_core(&self, core: &SqlSelectCore) -> Value {
        let core_projection: Value = core
            .projection
            .iter()
            .map(|x| match x {
                SqlProjection::All { collection } => {
                    json!({
                        "type": "All",
                        "collection": collection.as_ref().map(|token| token.lexeme.to_owned())
                    })
                }
                SqlProjection::Expr { expr, alias } => {
                    json!({
                        "type": "Expr",
                        "expr": self.visit_sql_expr(&expr),
                        "alias": alias.as_ref().map(|token| token.lexeme.to_owned())
                    })
                }
            })
            .collect();

        json!({
            "projection": core_projection,
            "from": core.from.as_ref().map(|x| self.visit_sql_subquery(&x))
        })
    }

    fn visit_sql_subquery(&self, subquery: &SqlCollectionSubquery) -> Value {
        match subquery {
            SqlCollectionSubquery::Collection {
                namespace,
                name,
                alias,
            } => {
                json!({
                    "type": "Collection",
                    "namespace": namespace.as_ref().map(|token| token.lexeme.to_owned()),
                    "name": name.lexeme,
                    "alias": alias.as_ref().map(|token| token.lexeme.to_owned())
                })
            }
            SqlCollectionSubquery::Group(groups) => {
                let subqueries: Value = groups.iter().map(|x| self.visit_sql_subquery(x)).collect();
                json!({
                    "type": "Group",
                    "subqueries": subqueries
                })
            }
            SqlCollectionSubquery::Select { expr, alias } => {
                json!({
                    "type": "Select",
                    "expr": self.visit_expr(*expr).unwrap(),
                    "alias": alias.as_ref().map(|token| token.lexeme.to_owned())
                })
            }
            SqlCollectionSubquery::Join(join_subquery, joins) => {
                let joins_ser: Value = joins
                    .iter()
                    .map(|x| {
                        json!({
                            "type": format!("{:?}", x.join_type),
                            "subquery": self.visit_sql_subquery(&x.subquery),
                            "constraint": x.join_constraint.as_ref().map(|y| {
                                self.visit_sql_expr(&y)
                            })
                        })
                    })
                    .collect();
                json!({
                    "type": "Join",
                    "subquery": self.visit_sql_subquery(&join_subquery),
                    "joins": joins_ser
                })
            }
        }
    }
}

impl<'a> ImmutableVisitor<Value, ()> for ProgramSerializer<'a> {
    fn visit_select(&self, select: &SqlSelect) -> Result<Value, ()> {
        let core = self.visit_sql_select_core(&select.core);

        let compound: Value = select
            .compound
            .iter()
            .map(|x| {
                json!({
                    "core": self.visit_sql_select_core(&x.core),
                    "operation": format!("{:?}", x.operator),
                })
            })
            .collect();

        let order_by: Option<Value> = select.order_by.as_ref().map(|x| {
            x.iter()
                .map(|order| {
                    let expr = self.visit_sql_expr(&order.expr);
                    let val = json!({
                        "expr": expr,
                        "ordering": format!("{:?}", order.ordering),
                    });
                    val
                })
                .collect()
        });

        let limit: Option<Value> = select.limit.as_ref().map(|x| {
            let count_part = self.visit_sql_expr(&x.count);

            let offset_part = if x.offset.is_some() {
                self.visit_sql_expr(&x.offset.as_ref().unwrap())
            } else {
                json!(serde_json::Value::Null)
            };

            json!({
                "count": count_part,
                "offset": offset_part
            })
        });

        Ok(json!({
            "core": core,
            "compound": compound,
            "order_by": order_by,
            "limit": limit
        }))
    }

    fn visit_expr(&self, eidx: ExprId) -> Result<Value, ()> {
        // TODO: Remove clone here
        let a = Rc::clone(&self.program.arena);
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
                operation,
                expr,
                span: _,
            } => {
                json!({
                    "type": "Expr::Unary",
                    "operation": operation,
                    "expr": self.visit_expr(*expr)?,
                })
            }
            Expr::Binary {
                operation,
                left,
                right,
                span: _,
            } => {
                json!({
                    "type": "Expr::Binary",
                    "left": self.visit_expr(*left)?,
                    "operation": operation,
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
                operation,
                right,
                span: _,
            } => {
                json!({
                    "type": "Expr::Logical",
                    "left": self.visit_expr(*left)?,
                    "operation": operation,
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
        let a = Rc::clone(&self.program.arena);
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
