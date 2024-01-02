use serde_json::{json, Value};

use crate::lang::ast::Literal;

use super::{
    ast::{
        expr::{Expr, ExprId},
        sql::{SqlCollectionSubquery, SqlExpr, SqlProjection, SqlSelect, SqlSelectCore, SqlCollectionIdentifier, SqlValues},
        stmt::{Stmt, StmtId},
        SqlVisitor, Visitor,
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

fn ser_collection_identifier(ident: &SqlCollectionIdentifier) -> Value {
    json!({
        "type": "Collection",
        "namespace": ident.namespace.as_ref().map(|token| token.lexeme.to_owned()),
        "name": ident.name.lexeme,
        "alias": ident.alias.as_ref().map(|token| token.lexeme.to_owned())
    })
}

impl<'a> SqlVisitor<Value, ()> for ProgramSerializer<'a> {
    fn visit_sql_expr(&self, sql_expr: &SqlExpr) -> Result<Value, ()> {
        let SqlExpr::Default(eidx) = sql_expr;
        self.visit_expr(*eidx)
    }

    fn visit_sql_select_core(&self, core: &SqlSelectCore) -> Result<Value, ()> {
        let core_distinct = json!(format!("{:?}", core.distinct));

        let core_projection: Result<Value, ()> = core
            .projection
            .iter()
            .map(|x| match x {
                SqlProjection::All { collection } => Ok(json!({
                    "type": "All",
                    "collection": collection.as_ref().map(|token| token.lexeme.to_owned())
                })),
                SqlProjection::Expr { expr, alias } => Ok(json!({
                    "type": "Expr",
                    "expr": self.visit_sql_expr(&expr)?,
                    "alias": alias.as_ref().map(|token| token.lexeme.to_owned())
                })),
            })
            .collect();

        let core_from = core
            .from
            .as_ref()
            .map(|x| self.visit_sql_subquery(&x))
            .unwrap_or_else(|| Ok(json!(serde_json::Value::Null)));

        let core_where = core
            .r#where
            .as_ref()
            .map(|x| self.visit_sql_expr(&x))
            .unwrap_or_else(|| Ok(json!(serde_json::Value::Null)));

        let core_group_by = core
            .group_by
            .as_ref()
            .map(|x| x.iter().map(|expr| self.visit_sql_expr(&expr)).collect())
            .unwrap_or_else(|| Ok(json!(serde_json::Value::Null)));

        let having = core
            .having
            .as_ref()
            .map(|x| self.visit_sql_expr(&x))
            .unwrap_or_else(|| Ok(json!(serde_json::Value::Null)));

        Ok(json!({
            "distinct": core_distinct,
            "projection": core_projection?,
            "from": core_from?,
            "where": core_where?,
            "group_by": core_group_by?,
            "having": having?
        }))
    }

    fn visit_sql_subquery(&self, subquery: &SqlCollectionSubquery) -> Result<Value, ()> {
        match subquery {
            SqlCollectionSubquery::Collection (ident) => Ok(ser_collection_identifier(ident)),
            SqlCollectionSubquery::Group(groups) => {
                let subqueries: Result<Value, ()> = groups
                    .iter()
                    .map(|x| Ok(self.visit_sql_subquery(x)?))
                    .collect();
                Ok(json!({
                    "type": "Group",
                    "subqueries": subqueries?
                }))
            }
            SqlCollectionSubquery::Select { expr, alias } => Ok(json!({
                "type": "Select",
                "expr": self.visit_expr(*expr)?,
                "alias": alias.as_ref().map(|token| token.lexeme.to_owned())
            })),
            SqlCollectionSubquery::Join(join_subquery, joins) => {
                let joins_ser: Result<Value, ()> = joins
                    .iter()
                    .map(|x| {
                        Ok(json!({
                            "type": format!("{:?}", x.join_type),
                            "subquery": self.visit_sql_subquery(&x.subquery)?,
                            "constraint": x.join_constraint.as_ref().map(|y| {
                                self.visit_sql_expr(&y)
                            }).unwrap_or_else(|| Ok(json!(serde_json::Value::Null)))?
                        }))
                    })
                    .collect();
                Ok(json!({
                    "type": "Join",
                    "subquery": self.visit_sql_subquery(&join_subquery)?,
                    "joins": joins_ser?
                }))
            }
        }
    }

    fn visit_sql_select(&self, select: &SqlSelect) -> Result<Value, ()> {
        let core = self.visit_sql_select_core(&select.core);

        let compound: Result<Value, ()> = select
            .compound
            .iter()
            .map(|x| {
                Ok(json!({
                    "core": self.visit_sql_select_core(&x.core)?,
                    "operation": format!("{:?}", x.operator),
                }))
            })
            .collect();

        let order_by: Result<Value, ()> = select
            .order_by
            .as_ref()
            .map(|x| {
                x.iter()
                    .map(|order| {
                        let expr = self.visit_sql_expr(&order.expr)?;
                        Ok(json!({
                            "expr": expr,
                            "ordering": format!("{:?}", order.ordering),
                        }))
                    })
                    .collect()
            })
            .unwrap_or_else(|| Ok(json!(serde_json::Value::Null)));

        let limit: Result<Value, ()> = select
            .limit
            .as_ref()
            .map(|x| {
                let count_part = self.visit_sql_expr(&x.count)?;

                let offset_part = if x.offset.is_some() {
                    self.visit_sql_expr(&x.offset.as_ref().unwrap())?
                } else {
                    json!(serde_json::Value::Null)
                };

                Ok(json!({
                    "count": count_part,
                    "offset": offset_part
                }))
            })
            .unwrap_or_else(|| Ok(json!(serde_json::Value::Null)));

        Ok(json!({
            "core": core?,
            "compound": compound?,
            "order_by": order_by?,
            "limit": limit?
        }))
    }

    fn visit_sql_insert(&self, sql_insert: &super::ast::sql::SqlInsert) -> Result<Value, ()> {
        let values = match &sql_insert.values {
            SqlValues::Values(values) => {
                let values_ser: Result<Value, ()> = values
                    .iter()
                    .map(|x| self.visit_sql_expr(&x))
                    .collect();
                Ok(json!({
                    "type": "Values",
                    "values": values_ser?
                }))
            }
            SqlValues::Select(select) => {
                Ok(json!({
                    "type": "Select",
                    "select": self.visit_sql_select(&select)?
                }))
            }
        };
        let ident = ser_collection_identifier(&sql_insert.collection);

        Ok(json!({
            "collection": ident,
            "values": values?,
        }))

    }

    fn visit_sql_update(&self, sql_update: &super::ast::sql::SqlUpdate) -> Result<Value, ()> {
        let ident = ser_collection_identifier(&sql_update.collection);

        let assignments: Result<Value, ()> = sql_update
            .assignments
            .iter()
            .map(|x| self.visit_sql_expr(&x))
            .collect();

        let r#where = sql_update
            .r#where
            .as_ref()
            .map(|x| self.visit_sql_expr(&x))
            .unwrap_or_else(|| Ok(json!(serde_json::Value::Null)));

        Ok(json!({
            "collection": ident,
            "assignments": assignments,
            "where": r#where?,
        }))
    }

    fn visit_sql_delete(&self, sql_delete: &super::ast::sql::SqlDelete) -> Result<Value, ()> {
        let ident = ser_collection_identifier(&sql_delete.collection);

        let r#where = sql_delete
            .r#where
            .as_ref()
            .map(|x| self.visit_sql_expr(&x))
            .unwrap_or_else(|| Ok(json!(serde_json::Value::Null)));

        Ok(json!({
            "collection": ident,
            "where": r#where?,
        }))
    }

}

impl<'a> Visitor<Value, ()> for ProgramSerializer<'a> {
    fn visit_expr(&self, eidx: ExprId) -> Result<Value, ()> {
        // TODO: Remove clone here
        let a = Rc::clone(&self.program.arena);
        let e = a.get_expression(eidx);

        let matched: Value = match e {
            Expr::Select { span: _, query } => json!({
                "type": "Expr::Select",
                "value": self.visit_sql_select(query)?,
            }),
            Expr::Insert { command, span } => {
                json!({
                    "type": "Expr::Insert",
                    "command": self.visit_sql_insert(command)?,
                })
            },
            Expr::Update { command, span } => {
                json!({
                    "type": "Expr::Update",
                    "command": self.visit_sql_update(command)?,
                })
            },
            Expr::Delete { command, span } => {
                json!({
                    "type": "Expr::Delete",
                    "command": self.visit_sql_delete(command)?,
                })
            },
            Expr::Literal {
                raw,
                span: _,
                value,
            } => {
                json!({
                    "type": "Expr::Literal",
                    "value": match value {
                        Literal::Object(map) => {
                            json!({
                                "type": "Object",
                                "value": map.keys().map(|item| json!({
                                    "key": item,
                                    "value": self.visit_expr(*map.get(item).unwrap()).unwrap()
                                })).collect::<Vec<_>>(),
                            })
                        }
                        Literal::Array(items) => {
                            json!({
                                "type": "Array",
                                "value": items.iter().map(|item| self.visit_expr(*item).unwrap()).collect::<Vec<_>>(),
                            })
                        },
                        _ => json!(format!("{:?}", value))
                    },
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
