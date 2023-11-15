use std::{
    fmt::{Debug, Display, Formatter},
    rc::Rc,
};

use super::{
    ast::{Expr, ExprId, Stmt, StmtId},
    parser::Parsed,
};

fn indent(level: u32, str: &str, terminate: bool) -> String {
    if terminate {
        return format!(
            "{}{}└──{}",
            "\n".to_owned(),
            "│  ".repeat(level as usize).as_str(),
            str.to_string()
        );
    }
    format!(
        "{}{}├──{}",
        "\n".to_owned(),
        "│  ".repeat(level as usize).as_str(),
        str.to_string()
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
            Expr::Unary(tok, expr) => {
                let buf = format!(
                    "{}{}{}",
                    &indent(level, "Unary", false),
                    &indent(level + 1, &format!("{:?}", tok), false),
                    &self.visit_expr(*expr, level + 1)?,
                );
                buf
            }
            Expr::Binary(tok, left, right) => {
                format!(
                    "{}{}{}",
                    &indent(
                        level,
                        &format!("Binary ({})", tok.lexeme.as_ref().unwrap()),
                        false
                    ),
                    &self.visit_expr(*left, level + 1)?,
                    &self.visit_expr(*right, level + 1)?
                )
            }
            Expr::Variable(tok) => indent(
                level,
                &format!("Variable ({})", tok.lexeme.as_ref().unwrap()),
                false,
            ),
            Expr::Assignment(tok, expr) => {
                let buf = format!(
                    "{}{}",
                    &indent(
                        level + 1,
                        &format!("Assignment ({})", tok.lexeme.as_ref().unwrap()),
                        false
                    ),
                    &self.visit_expr(*expr, level + 1)?,
                );
                buf
            }
            Expr::Logical(left, tok, right) => {
                let mut buf = format!(
                    "{}{}{}",
                    &indent(level, "Logical", false),
                    &indent(level + 1, &format!("{:?}", tok), false),
                    &self.visit_expr(*left, level + 1)?,
                );
                buf.push_str(&self.visit_expr(*right, level + 1)?);
                buf
            }
            Expr::Call(callee, paren, arguments) => {
                let mut buf = format!(
                    "{}{}",
                    &indent(level, "Call", false),
                    &self.visit_expr(*callee, level + 1)?,
                );
                for arg in arguments {
                    buf.push_str(&self.visit_expr(*arg, level + 1)?);
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
                    &format!("Declaration ({})", tok.lexeme.as_ref().unwrap()),
                    false,
                );
                match &tok.lexeme {
                    Some(_) => buf.push_str(&self.visit_expr(*expr, level + 1)?),
                    _ => (),
                }
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
            Stmt::Function(tok, args, body) => {
                let mut buf = indent(
                    level,
                    &format!("FunctionDeclaration [{} (", tok.lexeme.as_ref().unwrap()),
                    false,
                );
                for arg in args {
                    buf.push_str(&format!("{},", arg.lexeme.as_ref().unwrap()));
                }
                buf.push_str(")]");
                for expr in body.as_ref() {
                    buf.push_str(&self.visit_stmt(*expr, level + 1)?);
                }
                Ok(buf)
            }
        }
    }

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = "Program".to_owned();
        for stmt in self.statements.as_ref().unwrap() {
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
