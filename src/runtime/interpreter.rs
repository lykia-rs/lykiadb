use super::eval::{coerce2number, eval_binary, is_value_truthy};
use super::resolver::Resolver;
use crate::lang::ast::{Expr, ExprId, ParserArena, Stmt, StmtId, Visitor};
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
            Ok(coerce2number(self.visit_expr(eidx)?))
        } else {
            Ok(RV::Bool(is_value_truthy(self.visit_expr(eidx)?)))
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
                .read_at(unwrapped, &name.lexeme.unwrap().to_owned())
        } else {
            self.root_env
                .borrow()
                .read(&name.lexeme.unwrap().to_owned())
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
            Expr::Unary(tok, expr) => self.eval_unary(tok, *expr),
            Expr::Binary(tok, left, right) => self.eval_binary(*left, *right, tok),
            Expr::Variable(tok) => self.look_up_variable(tok.clone(), eidx),
            Expr::Assignment(tok, expr) => {
                let distance = self.resolver.get_distance(eidx);
                let evaluated = self.visit_expr(*expr)?;
                let result = if let Some(distance_unv) = distance {
                    self.env.borrow_mut().assign_at(
                        distance_unv,
                        tok.lexeme.as_ref().unwrap(),
                        evaluated.clone(),
                    )
                } else {
                    self.root_env
                        .borrow_mut()
                        .assign(tok.lexeme.as_ref().unwrap().to_string(), evaluated.clone())
                };
                if result.is_err() {
                    return Err(result.err().unwrap());
                }
                if let Err(HaltReason::GenericError(msg)) = result {
                    return Err(runtime_err(&msg, tok.line));
                }
                Ok(evaluated)
            }
            Expr::Logical(left, tok, right) => {
                let is_true = is_value_truthy(self.visit_expr(*left)?);

                if (tok.tok_type == kw!(Or) && is_true) || (tok.tok_type == kw!(And) && !is_true) {
                    return Ok(RV::Bool(is_true));
                }

                Ok(RV::Bool(is_value_truthy(self.visit_expr(*right)?)))
            }
            Expr::Call(callee, paren, arguments) => {
                let eval = self.visit_expr(*callee)?;

                if let Callable(arity, callable) = eval {
                    if arity.is_some() && arity.unwrap() != arguments.len() {
                        return Err(runtime_err(
                            &format!(
                                "Function expects {} arguments, while provided {}.",
                                arity.unwrap(),
                                arguments.len()
                            ),
                            paren.line,
                        ));
                    }

                    let mut args_evaluated: Vec<RV> = vec![];

                    for arg in arguments.iter() {
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
                        paren.line,
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
            Stmt::Declaration(tok, expr) => match &tok.lexeme {
                Some(var_name) => {
                    let evaluated = self.visit_expr(*expr)?;
                    self.env
                        .borrow_mut()
                        .declare(var_name.to_string(), evaluated);
                }
                None => {
                    return Err(runtime_err("Variable name cannot be empty", tok.line));
                }
            },
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
                    return Err(runtime_err("Unexpected break statement", token.line));
                }
            }
            Stmt::Continue(token) => {
                if !self.set_loop_state(LoopState::Continue, None) {
                    return Err(runtime_err("Unexpected continue statement", token.line));
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
                let name = token.lexeme.as_ref().unwrap().to_string();
                let fun = Function::UserDefined {
                    name: name.clone(),
                    body: Rc::clone(body),
                    parameters: parameters
                        .iter()
                        .map(|x| x.lexeme.as_ref().unwrap().to_string())
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
    use std::rc::Rc;

    use crate::runtime::{tests::get_runtime, types::RV};

    #[test]
    fn test_loop_statements() {
        let code = "for (var $i = 0; $i < 10; $i = $i + 1) {
            {
                {
                    if ($i == 2) continue;
                    if ($i == 8) break;
                    print($i);
                }
            }
        }";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut().expect(RV::Num(0.0));
        out.borrow_mut().expect(RV::Num(1.0));
        out.borrow_mut().expect(RV::Num(3.0));
        out.borrow_mut().expect(RV::Num(4.0));
        out.borrow_mut().expect(RV::Num(5.0));
        out.borrow_mut().expect(RV::Num(6.0));
        out.borrow_mut().expect(RV::Num(7.0));
        out.borrow().assert();
    }

    #[test]
    fn test_loop_large_break() {
        let code = "var $q = 0;

        for (var $i = 0; $i < 10000000; $i = $i+1) {
            break;
            $q = $q + 1;
            print(\"Shouldn't be shown\");
        }
        
        {
            {
                {
                    {
                        {
                            {
                                {
                                    print($q);
                                }
                            }
                        }
                    }
                }
            }
        }";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut().expect(RV::Num(0.0));
        out.borrow().assert();
    }

    #[test]
    fn test_if() {
        let code = "var $a = 30;

        if ($a > 50) {
            print(\"> 50\");
        }
        else if ($a > 20) {
            print(\"50 > $a > 20\");
        }
        else {
            print(\"< 20\");
        }";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut()
            .expect(RV::Str(Rc::new("50 > $a > 20".to_string())));
        out.borrow().assert();
    }

    #[test]
    fn test_higher_order_0() {
        let code = 
        "fun f($x, $q) {
            $x($q);
        }
        
        fun g($q) {
            print($q);
        }
        
        for (var $i=0; $i<10; $i = $i + 1) {
            f(g, $i);
        }";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut().expect(RV::Num(0.0));
        out.borrow_mut().expect(RV::Num(1.0));
        out.borrow_mut().expect(RV::Num(2.0));
        out.borrow_mut().expect(RV::Num(3.0));
        out.borrow_mut().expect(RV::Num(4.0));
        out.borrow_mut().expect(RV::Num(5.0));
        out.borrow_mut().expect(RV::Num(6.0));
        out.borrow_mut().expect(RV::Num(7.0));
        out.borrow_mut().expect(RV::Num(8.0));
        out.borrow_mut().expect(RV::Num(9.0));
        out.borrow().assert();
    }

    #[test]
    fn test_high_order_1() {
        let code = 
        "fun makeCounter() {
            var $i = 0;
            fun count() {
                $i = $i + 1;
                print($i);
            }
        
            return count;
        }
        var $count = makeCounter();
        $count();
        $count();";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut().expect(RV::Num(1.0));
        out.borrow_mut().expect(RV::Num(2.0));
        out.borrow().assert();
    }

    #[test]
    fn test_blocks_0() {
        let code = 
        "var $a = \"global a\";
        var $b = \"global b\";
        var $c = \"global c\";
        {
           var $a = \"outer a\";
           var $b = \"outer b\";
           {
              var $a = \"inner a\";
              print($a);
              print($b);
              print($c);
           }
           print($a);
           print($b);
           print($c);
        }
        print($a);
        print($b);
        print($c);";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.borrow_mut()
            .expect(RV::Str(Rc::new("inner a".to_string())));
        out.borrow_mut()
            .expect(RV::Str(Rc::new("outer b".to_string())));
        out.borrow_mut()
            .expect(RV::Str(Rc::new("global c".to_string())));
        out.borrow_mut()
            .expect(RV::Str(Rc::new("outer a".to_string())));
        out.borrow_mut()
            .expect(RV::Str(Rc::new("outer b".to_string())));
        out.borrow_mut()
            .expect(RV::Str(Rc::new("global c".to_string())));
        out.borrow_mut()
            .expect(RV::Str(Rc::new("global a".to_string())));
        out.borrow_mut()
            .expect(RV::Str(Rc::new("global b".to_string())));
        out.borrow_mut()
            .expect(RV::Str(Rc::new("global c".to_string())));
        out.borrow().assert();
    }
}
