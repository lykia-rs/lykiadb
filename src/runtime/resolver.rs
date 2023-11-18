use crate::lang::ast::{Expr, ExprId, ParserArena, Stmt, StmtId, Visitor};
use crate::lang::token::Token;
use crate::runtime::types::RV;
use rustc_hash::FxHashMap;
use std::rc::Rc;

pub struct Resolver {
    scopes: Vec<FxHashMap<String, bool>>,
    locals: FxHashMap<usize, usize>,
    arena: Rc<ParserArena>,
}

#[derive(Debug)]
pub enum ResolveError {
    GenericError { token: Token, message: String },
}

impl Resolver {
    pub fn new(arena: Rc<ParserArena>) -> Resolver {
        Resolver {
            scopes: vec![],
            locals: FxHashMap::default(),
            arena,
        }
    }

    pub fn get_distance(&self, eid: ExprId) -> Option<usize> {
        self.locals.get(&eid).copied()
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn resolve_stmts(&mut self, statements: &Vec<StmtId>) {
        for statement in statements {
            self.resolve_stmt(*statement);
        }
    }

    pub fn resolve_stmt(&mut self, statement: StmtId) {
        self.visit_stmt(statement).unwrap();
    }

    pub fn resolve_expr(&mut self, expr: ExprId) {
        self.visit_expr(expr).unwrap();
    }

    pub fn resolve_local(&mut self, expr: ExprId, name: &Token) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name.span.lexeme.as_ref().to_string()) {
                self.locals.insert(expr, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    pub fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let last = self.scopes.last_mut();
        last.unwrap()
            .insert(name.span.lexeme.as_ref().to_string(), false);
    }

    pub fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let last = self.scopes.last_mut();
        last.unwrap()
            .insert(name.span.lexeme.as_ref().to_string(), true);
    }
}

impl Visitor<RV, ResolveError> for Resolver {
    fn visit_expr(&mut self, eidx: ExprId) -> Result<RV, ResolveError> {
        let a = Rc::clone(&self.arena);
        let e = a.get_expression(eidx);
        match e {
            Expr::Literal(_) => (),
            Expr::Grouping(expr) => self.resolve_expr(*expr),
            Expr::Unary(_tok, expr) => self.resolve_expr(*expr),
            Expr::Binary(_tok, left, right) => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            Expr::Variable(tok) => {
                if !self.scopes.is_empty() {
                    let last_scope = self.scopes.last().unwrap();
                    let value = last_scope.get(&tok.span.lexeme.to_string());
                    if value.is_some() && !(*value.unwrap()) {
                        return Err(ResolveError::GenericError {
                            token: tok.clone(),
                            message: "Can't read local variable in its own initializer."
                                .to_string(),
                        });
                    }
                }
                self.resolve_local(eidx, tok);
            }
            Expr::Assignment(name, value) => {
                self.resolve_expr(*value);
                self.resolve_local(eidx, name);
            }
            Expr::Logical(left, _tok, right) => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            Expr::Call(callee, _paren, arguments) => {
                self.resolve_expr(*callee);

                for argument in arguments {
                    self.resolve_expr(*argument);
                }
            }
            Expr::Select(_) => (),
        };
        Ok(RV::Undefined)
    }

    fn visit_stmt(&mut self, sidx: StmtId) -> Result<RV, ResolveError> {
        let a = Rc::clone(&self.arena);
        let s = a.get_statement(sidx);
        match s {
            Stmt::Break(_token) | Stmt::Continue(_token) => (),
            Stmt::Expression(expr) => {
                self.resolve_expr(*expr);
            }
            Stmt::Declaration(_tok, expr) => {
                self.declare(_tok);
                self.resolve_expr(*expr);
                self.define(_tok);
            }
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve_stmts(statements);
                self.end_scope();
            }
            Stmt::If(condition, if_stmt, else_optional) => {
                self.resolve_expr(*condition);
                self.resolve_stmt(*if_stmt);
                if else_optional.is_some() {
                    self.resolve_stmt(*else_optional.as_ref().unwrap());
                }
            }
            Stmt::Loop(condition, stmt, post_body) => {
                self.resolve_expr(*condition.as_ref().unwrap());
                self.resolve_stmt(*stmt);
                if post_body.is_some() {
                    self.resolve_stmt(*post_body.as_ref().unwrap());
                }
            }
            Stmt::Return(_token, expr) => {
                if expr.is_some() {
                    self.resolve_expr(expr.unwrap());
                }
            }
            Stmt::Function(_token, parameters, body) => {
                self.declare(_token);
                self.define(_token);
                self.begin_scope();
                for param in parameters {
                    self.declare(param);
                    self.define(param);
                }
                self.resolve_stmts(body.as_ref());
                self.end_scope();
            }
        }
        Ok(RV::Undefined)
    }
}

#[cfg(test)]
mod test {
    use crate::runtime::{tests::get_runtime, types::RV};
    use std::rc::Rc;

    #[test]
    fn test_resolving_read_0() {
        let code = "var $a = \"global\";
        {
          fun showA() {
            print($a);
          }
        
          showA();
          var $a = \"block\";
          showA();
        }";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut().expect(vec![
            RV::Str(Rc::new("global".to_string())),
            RV::Str(Rc::new("global".to_string())),
        ]);
    }

    #[test]
    fn test_resolving_read_1() {
        let code = "var $a = \"global\";
        {
            fun showA() {
                print($a);
            }

            showA();
            var $a = \"block\";
            showA();
            fun showB() {
                print($a);
            }
            showB();
        }";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut().expect(vec![
            RV::Str(Rc::new("global".to_string())),
            RV::Str(Rc::new("global".to_string())),
            RV::Str(Rc::new("block".to_string())),
        ]);
    }

    #[test]
    fn test_resolving_read_2() {
        let code = "{
            var $a = \"global\";
            {
              fun showA() {
                print($a);
              }
          
              showA();
              var $a = \"block\";
              showA();
            }
          }";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut().expect(vec![
            RV::Str(Rc::new("global".to_string())),
            RV::Str(Rc::new("global".to_string())),
        ]);
    }

    #[test]
    fn test_resolving_write_0() {
        let code = "var $a = \"global\";
        {
          fun showA() {
            print($a);
          }
        
          var $a = \"block\";
          
          fun showB() {
            print($a);
          }
        
          //
          showA();
          showB();
          //
          $a = \"test\";
          //
          showA();
          showB();
        }";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut().expect(vec![
            RV::Str(Rc::new("global".to_string())),
            RV::Str(Rc::new("block".to_string())),
            RV::Str(Rc::new("global".to_string())),
            RV::Str(Rc::new("test".to_string())),
        ]);
    }
}
