use super::eval::{coerce2number, eval_binary, is_value_truthy};
use super::resolver::Resolver;
use crate::lang::ast::expr::{Expr, ExprId};
use crate::lang::ast::stmt::{Stmt, StmtId};
use crate::lang::ast::{ParserArena, Visitor};
use crate::lang::token::Keyword::*;
use crate::lang::token::Symbol::*;
use crate::lang::token::Token;
use crate::lang::token::TokenType;
use crate::runtime::environment::Environment;
use crate::runtime::types::RV::Callable;
use crate::runtime::types::{Function, RV};
use crate::util::Shared;
use crate::{kw, sym};
use std::rc::Rc;
use std::vec;

#[derive(Debug)]
pub enum HaltReason {
    GenericError(String),
    Return(RV),
}

pub fn runtime_err(msg: &str, line: u32) -> HaltReason {
    HaltReason::GenericError(format!("{} at line {}", msg, line + 1))
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum LoopState {
    Go,
    Broken,
    Continue,
}

#[derive(Debug)]
pub struct Context {
    ongoing_loops: Option<Vec<LoopState>>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            ongoing_loops: None,
        }
    }

    pub fn push_loop(&mut self, state: LoopState) {
        if self.ongoing_loops.is_none() {
            self.ongoing_loops = Some(vec![]);
        }
        self.ongoing_loops.as_mut().unwrap().push(state);
    }

    pub fn pop_loop(&mut self) {
        self.ongoing_loops.as_mut().unwrap().pop();
    }

    pub fn is_loops_empty(&self) -> bool {
        if self.ongoing_loops.is_none() {
            return true;
        }
        return self.ongoing_loops.as_ref().unwrap().is_empty();
    }

    pub fn get_last_loop(&self) -> Option<&LoopState> {
        if self.ongoing_loops.is_none() {
            return None;
        }
        return self.ongoing_loops.as_ref().unwrap().last();
    }

    pub fn set_last_loop(&mut self, to: LoopState) {
        if self.ongoing_loops.is_none() {
            return;
        }
        self.pop_loop();
        self.push_loop(to);
    }
}

pub struct Interpreter {
    env: Shared<Environment>,
    root_env: Shared<Environment>,
    arena: Rc<ParserArena>,
    call_stack: Vec<Context>,
    resolver: Rc<Resolver>,
}

impl Interpreter {
    pub fn new(
        env: Shared<Environment>,
        arena: Rc<ParserArena>,
        resolver: Rc<Resolver>,
    ) -> Interpreter {
        Interpreter {
            env: env.clone(),
            root_env: env,
            arena: Rc::clone(&arena),
            call_stack: vec![Context::new()],
            resolver,
        }
    }

    fn eval_unary(&mut self, tok: &Token, eidx: ExprId) -> Result<RV, HaltReason> {
        if tok.tok_type == sym!(Minus) {
            if let Some(num) = coerce2number(self.visit_expr(eidx)?) {
                return Ok(RV::Num(-num));
            }
            Ok(RV::NaN)
        } else {
            Ok(RV::Bool(!is_value_truthy(self.visit_expr(eidx)?)))
        }
    }

    fn eval_binary(&mut self, lidx: ExprId, ridx: ExprId, tok: &Token) -> Result<RV, HaltReason> {
        let left_eval = self.visit_expr(lidx)?;
        let right_eval = self.visit_expr(ridx)?;

        Ok(eval_binary(left_eval, right_eval, tok))
    }

    fn look_up_variable(&self, name: Token, eid: ExprId) -> Result<RV, HaltReason> {
        let distance = self.resolver.get_distance(eid);
        if let Some(unwrapped) = distance {
            self.env
                .borrow()
                .read_at(unwrapped, &name.span.lexeme.to_owned())
        } else {
            self.root_env.borrow().read(&name.span.lexeme.to_owned())
        }
    }
}

impl Interpreter {
    fn is_loop_at(&self, state: LoopState) -> bool {
        let last_loop = *self.call_stack[0].get_last_loop().unwrap();
        last_loop == state
    }

    fn set_loop_state(&mut self, to: LoopState, from: Option<LoopState>) -> bool {
        if from.is_none() {
            return if !self.call_stack[0].is_loops_empty() {
                self.call_stack[0].set_last_loop(to);
                true
            } else {
                false
            };
        } else if self.is_loop_at(from.unwrap()) {
            self.call_stack[0].set_last_loop(to);
        }
        true
    }

    pub fn user_fn_call(
        &mut self,
        statements: &Vec<StmtId>,
        environment: Shared<Environment>,
    ) -> Result<RV, HaltReason> {
        self.execute_block(statements, Some(environment))
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<StmtId>,
        env_opt: Option<Shared<Environment>>,
    ) -> Result<RV, HaltReason> {
        let mut env_tmp: Option<Shared<Environment>> = None;

        if let Some(env_opt_unwrapped) = env_opt {
            env_tmp = Some(self.env.clone());
            self.env = env_opt_unwrapped;
        } else {
            self.env = Environment::new(Some(self.env.clone()));
        }
        let mut ret = Ok(RV::Undefined);

        for statement in statements {
            ret = self.visit_stmt(*statement);
            if ret.is_err() {
                break;
            }
        }
        if let Some(env_tmp_unwrapped) = env_tmp {
            self.env = env_tmp_unwrapped;
        } else {
            self.env = self.env.clone().borrow_mut().pop();
        }
        ret
    }
}

impl Visitor<RV, HaltReason> for Interpreter {
    fn visit_expr(&mut self, eidx: ExprId) -> Result<RV, HaltReason> {
        // TODO: Remove clone here
        let a = Rc::clone(&self.arena);
        let e = a.get_expression(eidx);
        match e {
            Expr::Select(val) => Ok(RV::Str(Rc::new(format!("{:?}", val)))),
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Grouping(expr) => self.visit_expr(*expr),
            Expr::Unary { token, expr } => self.eval_unary(token, *expr),
            Expr::Binary { token, left, right } => self.eval_binary(*left, *right, token),
            Expr::Variable(tok) => self.look_up_variable(tok.clone(), eidx),
            Expr::Assignment { var_tok, expr } => {
                let distance = self.resolver.get_distance(eidx);
                let evaluated = self.visit_expr(*expr)?;
                let result = if let Some(distance_unv) = distance {
                    self.env.borrow_mut().assign_at(
                        distance_unv,
                        var_tok.span.lexeme.as_ref(),
                        evaluated.clone(),
                    )
                } else {
                    self.root_env
                        .borrow_mut()
                        .assign(var_tok.span.lexeme.as_ref().to_string(), evaluated.clone())
                };
                if result.is_err() {
                    return Err(result.err().unwrap());
                }
                if let Err(HaltReason::GenericError(msg)) = result {
                    return Err(runtime_err(&msg, var_tok.span.line));
                }
                Ok(evaluated)
            }
            Expr::Logical { left, token, right } => {
                let is_true = is_value_truthy(self.visit_expr(*left)?);

                if (token.tok_type == kw!(Or) && is_true)
                    || (token.tok_type == kw!(And) && !is_true)
                {
                    return Ok(RV::Bool(is_true));
                }

                Ok(RV::Bool(is_value_truthy(self.visit_expr(*right)?)))
            }
            Expr::Call {
                callee,
                paren,
                args,
            } => {
                let eval = self.visit_expr(*callee)?;

                if let Callable(arity, callable) = eval {
                    if arity.is_some() && arity.unwrap() != args.len() {
                        return Err(runtime_err(
                            &format!(
                                "Function expects {} arguments, while provided {}.",
                                arity.unwrap(),
                                args.len()
                            ),
                            paren.span.line,
                        ));
                    }

                    let mut args_evaluated: Vec<RV> = vec![];

                    for arg in args.iter() {
                        args_evaluated.push(self.visit_expr(*arg)?);
                    }
                    self.call_stack.insert(0, Context::new());

                    let val = callable.call(self, args_evaluated.as_slice());
                    self.call_stack.remove(0);
                    match val {
                        Err(HaltReason::Return(ret_val)) => Ok(ret_val),
                        Ok(unpacked_val) => Ok(unpacked_val),
                        other_err @ Err(_) => other_err,
                    }
                } else {
                    Err(runtime_err(
                        "Expression does not yield a callable",
                        paren.span.line,
                    ))
                }
            }
        }
    }

    fn visit_stmt(&mut self, sidx: StmtId) -> Result<RV, HaltReason> {
        if !self.call_stack[0].is_loops_empty()
            && *self.call_stack[0].get_last_loop().unwrap() != LoopState::Go
        {
            return Ok(RV::Undefined);
        }
        // TODO: Remove clone here
        let a = Rc::clone(&self.arena);
        let s = a.get_statement(sidx);
        match s {
            Stmt::Expression(expr) => {
                return self.visit_expr(*expr);
            }
            Stmt::Declaration(tok, expr) => {
                let evaluated = self.visit_expr(*expr)?;
                self.env
                    .borrow_mut()
                    .declare(tok.span.lexeme.to_string(), evaluated);
            }
            Stmt::Block(statements) => {
                return self.execute_block(statements, None);
            }
            Stmt::If(condition, if_stmt, else_optional) => {
                if is_value_truthy(self.visit_expr(*condition)?) {
                    self.visit_stmt(*if_stmt)?;
                } else if let Some(else_stmt) = else_optional {
                    self.visit_stmt(*else_stmt)?;
                }
            }
            Stmt::Loop(condition, stmt, post_body) => {
                self.call_stack[0].push_loop(LoopState::Go);
                while !self.is_loop_at(LoopState::Broken)
                    && (condition.is_none()
                        || is_value_truthy(self.visit_expr(condition.unwrap())?))
                {
                    self.visit_stmt(*stmt)?;
                    self.set_loop_state(LoopState::Go, Some(LoopState::Continue));
                    if let Some(post) = post_body {
                        self.visit_stmt(*post)?;
                    }
                }
                self.call_stack[0].pop_loop();
            }
            Stmt::Break(token) => {
                if !self.set_loop_state(LoopState::Broken, None) {
                    return Err(runtime_err("Unexpected break statement", token.span.line));
                }
            }
            Stmt::Continue(token) => {
                if !self.set_loop_state(LoopState::Continue, None) {
                    return Err(runtime_err(
                        "Unexpected continue statement",
                        token.span.line,
                    ));
                }
            }
            Stmt::Return(_token, expr) => {
                if expr.is_some() {
                    let ret = self.visit_expr(expr.unwrap())?;
                    return Err(HaltReason::Return(ret));
                }
                return Err(HaltReason::Return(RV::Undefined));
            }
            Stmt::Function(token, parameters, body) => {
                let name = token.span.lexeme.as_ref().to_string();
                let fun = Function::UserDefined {
                    name: name.clone(),
                    body: Rc::clone(body),
                    parameters: parameters
                        .iter()
                        .map(|x| x.span.lexeme.as_ref().to_string())
                        .collect(),
                    closure: self.env.clone(),
                };

                self.env
                    .borrow_mut()
                    .declare(name, Callable(Some(parameters.len()), fun.into()));
            }
        }
        Ok(RV::Undefined)
    }
}

#[cfg(test)]
mod test {
    use crate::runtime::{tests::get_runtime, types::RV};

    #[test]
    fn test_unary_evaluation() {
        let code = "
            print(-2);
            print(-(-2));
            print(!3);
            print(!!3);
            print(!!!3);
        ";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut().expect(vec![
            RV::Num(-2.0),
            RV::Num(2.0),
            RV::Bool(false),
            RV::Bool(true),
            RV::Bool(false),
        ]);
    }
    #[test]
    fn test_binary_evaluation() {
        let code = "
            print(5-(-2));
            print((5 + 2) * 4);
            print(5 + 2 * 4);
            print((13 + 4) * (7 + 3));
            print(-5-2);
        ";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut().expect(vec![
            RV::Num(7.0),
            RV::Num(28.0),
            RV::Num(13.0),
            RV::Num(170.0),
            RV::Num(-7.0),
        ]);
    }

    #[test]
    fn test_logical_evaluation() {
        let code = "
            print(5 and 1);
            print(5 or 1);
            print(5 and 0);
            print(5 or 0);
            print(!(5 or 0));
            print(!(5 or 0) or 1);
            print(!(5 or 0) or (1 and 0));
        ";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut().expect(vec![
            RV::Bool(true),
            RV::Bool(true),
            RV::Bool(false),
            RV::Bool(true),
            RV::Bool(false),
            RV::Bool(true),
            RV::Bool(false),
        ]);
    }
}
