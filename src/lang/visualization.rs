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
        Ok(indent(level, &format!("{:?}", self.arena.get_expression(eidx)), false))
    }

    fn visit_stmt(&self, sidx: StmtId, level: u32) -> Result<String, ()> {
         // TODO: Remove clone here
         let a = Rc::clone(&self.arena);
         let s = a.get_statement(sidx);
         match s {
            Stmt::Expression(expr) => {
                return Ok(self.visit_expr(*expr, level + 1)?);
            },
            Stmt::Declaration(tok, expr) => {
                match &tok.lexeme {
                    Some(var_name) => {
                        let evaluated = self.visit_expr(*expr, level + 1);
                    },
                    _ => ()
                }
            },
            Stmt::Block(statements) => { 
                let mut buf = String::new();
                buf.push_str(&indent(level, "Block", false));
                for statement in statements {
                    buf.push_str(&self.visit_stmt(*statement, level + 1)?);
                }
                return Ok(buf);
            },
            Stmt::If(condition, if_stmt, else_optional) => {
                let mut buf = String::new();
                buf.push_str(&indent(level, "If", false));
                buf.push_str(&self.visit_expr(*condition, level + 1)?);
                buf.push_str(&self.visit_stmt(*if_stmt, level + 1)?);
                if let Some(else_stmt) = else_optional {
                    buf.push_str(&indent(level, &self.visit_stmt(*else_stmt, level + 1)?, false));
                }
                buf.push_str(&indent(level, "", false));
                return Ok(buf)
            },
            Stmt::Loop(condition, stmt, post_body) => {
                let mut buf = String::new();
                buf.push_str(&indent(level, "Loop", false));
                buf.push_str(&self.visit_expr(*condition.as_ref().unwrap(), level + 1)?);
                buf.push_str(&self.visit_stmt(*stmt, level + 1)?);
                buf.push_str(&self.visit_stmt(*post_body.as_ref().unwrap(), level + 1)?);
                return Ok(buf);
            },
            Stmt::Break(token) => {
                return Ok(indent(level, &"Break".to_string(), false));
            },
            Stmt::Continue(token) => {
                return Ok(indent(level, &"Continue".to_string(), false));
            },
            Stmt::Return(_token, expr) => {
                if expr.is_some() {
                    return Ok(self.visit_expr(expr.unwrap(), level + 1)?);
                }
                return Ok("Return".to_owned());
            },
            Stmt::Function(token, parameters, body) => {
                let name = token.lexeme.as_ref().unwrap().to_string();
                /*
                let fun = Function::UserDefined {
                    name: name.clone(),
                    body: Rc::clone(body),
                    parameters:parameters.into_iter().map(|x| x.lexeme.as_ref().unwrap().to_string()).collect(),
                    closure: self.env.clone(),
                }; */
            },
        }
         return Ok(format!("{:?}", s));
    }

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        buf.push_str("Program");
        for stmt in self.statements.as_ref().unwrap() {
            let result = self.visit_stmt(*stmt, 0).unwrap();
            buf.push_str(&result);
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