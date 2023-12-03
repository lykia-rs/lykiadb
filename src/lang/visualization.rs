use std::{
    fmt::{Debug, Display, Formatter},
    rc::Rc,
};

use super::parser::Parsed;
use crate::lang::ast::expr::{Expr, ExprId};
use crate::lang::ast::stmt::{Stmt, StmtId};

fn indent(level: u32, str: &str, terminate: bool) -> String {
    if terminate {
        return format!(
            "{}{}└──{}",
            "\n".to_owned(),
            "│  ".repeat(level as usize).as_str(),
            str
        );
    }
    format!(
        "{}{}├──{}",
        "\n".to_owned(),
        "│  ".repeat(level as usize).as_str(),
        str
    )
}

impl Parsed {
    fn visit_expr(&self, eidx: ExprId, level: u32) -> Result<String, ()> {
        // TODO: Remove clone here
        let a = Rc::clone(&self.arena);
        let e = a.get_expression(eidx);

        let matched: String = match e {
            Expr::Select(val) => format!("Select ({:?})", val),
            Expr::Literal(val) => indent(level, &format!("Literal ({:?})", val), true),
            Expr::Grouping(expr) => self.visit_expr(*expr, level + 1)?,
            Expr::Unary { token, expr } => {
                let buf = format!(
                    "{}{}{}",
                    &indent(level, "Unary", false),
                    &indent(level + 1, &format!("{:?}", token), false),
                    &self.visit_expr(*expr, level + 1)?,
                );
                buf
            }
            Expr::Binary { token, left, right } => {
                format!(
                    "{}{}{}",
                    &indent(
                        level,
                        &format!("Binary ({})", token.span.lexeme.as_ref()),
                        false
                    ),
                    &self.visit_expr(*left, level + 1)?,
                    &self.visit_expr(*right, level + 1)?
                )
            }
            Expr::Variable(tok) => indent(
                level,
                &format!("Variable ({})", tok.span.lexeme.as_ref()),
                false,
            ),
            Expr::Assignment { var_tok, expr } => {
                let buf = format!(
                    "{}{}",
                    &indent(
                        level + 1,
                        &format!("Assignment ({})", var_tok.span.lexeme.as_ref()),
                        false
                    ),
                    &self.visit_expr(*expr, level + 1)?,
                );
                buf
            }
            Expr::Logical { left, token, right } => {
                let mut buf = format!(
                    "{}{}{}",
                    &indent(level, "Logical", false),
                    &indent(level + 1, &format!("{:?}", token), false),
                    &self.visit_expr(*left, level + 1)?,
                );
                buf.push_str(&self.visit_expr(*right, level + 1)?);
                buf
            }
            Expr::Call {
                callee,
                paren: _,
                args,
            } => {
                let mut buf = format!(
                    "{}{}",
                    &indent(level, "Call", false),
                    &self.visit_expr(*callee, level + 1)?,
                );
                for arg in args {
                    buf.push_str(&self.visit_expr(*arg, level + 1)?);
                }
                buf
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
                let mut buf = indent(level, &format!("FunctionDeclaration [{} (", fn_name), false);
                for param in parameters {
                    buf.push_str(&format!("{},", param.span.lexeme.as_ref()));
                }
                buf.push_str(")]");
                for stmt in body.as_ref() {
                    buf.push_str(&self.visit_stmt(*stmt, level + 1)?);
                }
                buf
            }
        };

        Ok(matched)
    }

    fn visit_stmt(&self, sidx: StmtId, level: u32) -> Result<String, ()> {
        // TODO: Remove clone here
        let a = Rc::clone(&self.arena);
        let s = a.get_statement(sidx);
        match s {
            Stmt::Expression(expr) => {
                let mut buf = indent(level, "ExprStmt", false);
                buf.push_str(&self.visit_expr(*expr, level + 1)?);
                Ok(buf)
            }
            Stmt::Declaration(tok, expr) => {
                let mut buf = indent(
                    level,
                    &format!("Declaration ({})", tok.span.lexeme.as_ref()),
                    false,
                );
                buf.push_str(&self.visit_expr(*expr, level + 1)?);
                Ok(buf)
            }
            Stmt::Block(statements) => {
                let mut buf = indent(level, "Block", false);
                for statement in statements {
                    buf.push_str(&self.visit_stmt(*statement, level + 1)?);
                }
                Ok(buf)
            }
            Stmt::If(condition, if_stmt, else_optional) => {
                let mut buf = format!(
                    "{}{}{}",
                    &indent(level, "If", false),
                    &self.visit_expr(*condition, level + 1)?,
                    &self.visit_stmt(*if_stmt, level + 1)?,
                );
                if let Some(else_stmt) = else_optional {
                    buf.push_str(&indent(
                        level,
                        &format!("Else {}", &self.visit_stmt(*else_stmt, level + 1)?),
                        false,
                    ));
                }
                buf.push_str(&indent(level, "", false));
                Ok(buf)
            }
            Stmt::Loop(condition, stmt, post_body) => Ok(format!(
                "{}{}{}{}",
                &indent(level, "Loop", false),
                &self.visit_expr(*condition.as_ref().unwrap(), level + 1)?,
                &self.visit_stmt(*stmt, level + 1)?,
                &self.visit_stmt(*post_body.as_ref().unwrap(), level + 1)?
            )),
            Stmt::Break(_) => Ok(indent(level, "Break", false)),
            Stmt::Continue(_) => Ok(indent(level, "Continue", false)),
            Stmt::Return(_, expr) => {
                let mut buf = indent(level, "Return", false);
                if expr.is_some() {
                    buf.push_str(&self.visit_expr(expr.unwrap(), level + 1)?);
                }
                Ok(buf)
            }
        }
    }

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = "Program".to_owned();
        for stmt in &self.statements {
            buf.push_str(&self.visit_stmt(*stmt, 0).unwrap());
        }
        writeln!(f, "{}", buf)?;
        Ok(())
    }
}

impl Display for Parsed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl Debug for Parsed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}
