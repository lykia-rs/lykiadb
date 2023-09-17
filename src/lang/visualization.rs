use std::{rc::Rc, fmt::{Debug, Display, Formatter}};

use super::{parser::Parsed, ast::{ExprId, StmtId, Stmt}};

fn indent(level: u32, str: &str, terminate: bool) -> String {
    if terminate {
        return format!("{}{}└──{}", "\n".to_owned(), "│  ".repeat(level as usize).as_str(), str.to_string());
    }
    format!("{}{}├──{}", "\n".to_owned(), "│  ".repeat(level as usize).as_str(), str.to_string())
}

impl Parsed {
    
    fn visit_expr(&self, eidx: ExprId, level: u32) -> Result<String, ()> {
        Ok(indent(level, &format!("{:?}", self.arena.get_expression(eidx)), true))
    }

    fn visit_stmt(&self, sidx: StmtId, level: u32) -> Result<String, ()> {
         // TODO: Remove clone here
         let a = Rc::clone(&self.arena);
         let s = a.get_statement(sidx);
         match s {
            Stmt::Expression(expr) => {
                let mut buf = indent(level, "ExprStmt", false);
                buf.push_str(&self.visit_expr(*expr, level + 1)?);
                return Ok(buf);
            },
            Stmt::Declaration(tok, expr) => {
                let mut buf = indent(level, &format!("Declaration ({})", tok.lexeme.as_ref().unwrap()), false);
                match &tok.lexeme {
                    Some(_) => buf.push_str(&self.visit_expr(*expr, level + 1)?),
                    _ => ()
                }
                return Ok(buf);
            },
            Stmt::Block(statements) => { 
                let mut buf = indent(level, "Block", false);                
                for statement in statements {
                    buf.push_str(&self.visit_stmt(*statement, level + 1)?);
                }
                return Ok(buf);
            },
            Stmt::If(condition, if_stmt, else_optional) => {
                let mut buf = format!("{}{}{}",
                    &indent(level, "If", false),
                    &self.visit_expr(*condition, level + 1)?,
                    &self.visit_stmt(*if_stmt, level + 1)?,
                );
                if let Some(else_stmt) = else_optional {
                    buf.push_str(&indent(level, &self.visit_stmt(*else_stmt, level + 1)?, false));
                }
                buf.push_str(&indent(level, "", false));
                return Ok(buf)
            },
            Stmt::Loop(condition, stmt, post_body) => {
                return Ok(format!("{}{}{}{}", 
                    &indent(level, "Loop", false),
                    &self.visit_expr(*condition.as_ref().unwrap(), level + 1)?,
                    &self.visit_stmt(*stmt, level + 1)?,
                    &self.visit_stmt(*post_body.as_ref().unwrap(), level + 1)?
                ));
            },
            Stmt::Break(_) => {
                return Ok(indent(level, &"Break".to_string(), false));
            },
            Stmt::Continue(_) => {
                return Ok(indent(level, &"Continue".to_string(), false));
            },
            Stmt::Return(_, expr) => {
                let mut buf = indent(level, "Return", false);                
                if expr.is_some() {
                    buf.push_str(&self.visit_expr(expr.unwrap(), level + 1)?);
                }
                return Ok(buf);
            },
            Stmt::Function(tok, args, body) => {
                let mut buf = indent(level, &format!("FunctionDeclaration [{} (", tok.lexeme.as_ref().unwrap()), false);
                for arg in args {
                    buf.push_str(arg.lexeme.as_ref().unwrap())
                }
                buf.push_str(")]");
                for expr in body.as_ref() {
                    buf.push_str(&self.visit_stmt(*expr, level + 1)?);
                }
                return Ok(buf);
            },
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