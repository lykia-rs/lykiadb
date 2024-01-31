use rustc_hash::FxHashMap;

use super::environment::{EnvId, Environment};
use super::eval::eval_binary;
use super::resolver::Resolver;
use super::types::Stateful;
use crate::lang::ast::expr::{Expr, ExprId, Operation};
use crate::lang::ast::program::AstArena;
use crate::lang::ast::stmt::{Stmt, StmtId};
use crate::lang::Literal;

use crate::lang::ast::visitor::VisitorMut;
use crate::lang::tokens::token::Span;
use crate::lang::tokens::token::Spanned;
use crate::runtime::types::RV::Callable;
use crate::runtime::types::{Function, RV};
use crate::util::alloc_shared;

use std::sync::Arc;
use std::vec;

#[derive(Debug, Clone)]
pub enum InterpretError {
    NotCallable {
        span: Span,
    },
    ArityMismatch {
        span: Span,
        expected: usize,
        found: usize,
    },
    UnexpectedStatement {
        span: Span,
    },
    PropertyNotFound {
        span: Span,
        property: String,
    },
    /*AssignmentToUndefined {
        token: Token,
    },
    VariableNotFound {
        token: Token,
    },*/
    Other {
        message: String,
    }, // TODO(vck): Refactor this
}

#[derive(Debug)]
pub enum HaltReason {
    Error(InterpretError),
    Return(RV),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum LoopState {
    Go,
    Broken,
    Continue,
    Function,
}

struct LoopStack {
    ongoing_loops: Vec<LoopState>,
}

impl LoopStack {
    pub fn new() -> LoopStack {
        LoopStack {
            ongoing_loops: vec![],
        }
    }

    pub fn push_fn(&mut self) {
        self.ongoing_loops.push(LoopState::Function);
    }

    pub fn pop_fn(&mut self) {
        if self.ongoing_loops.last() == Some(&LoopState::Function) {
            self.ongoing_loops.pop();
        }
    }

    pub fn push_loop(&mut self, state: LoopState) {
        self.ongoing_loops.push(state);
    }

    pub fn pop_loop(&mut self) {
        if self.is_loops_empty() {
            return;
        }
        self.ongoing_loops.pop();
    }

    pub fn is_loops_empty(&self) -> bool {
        self.ongoing_loops.is_empty() || self.ongoing_loops.last() == Some(&LoopState::Function)
    }

    pub fn get_last_loop(&self) -> Option<&LoopState> {
        if self.is_loops_empty() {
            return None;
        }
        return self.ongoing_loops.last();
    }

    pub fn set_last_loop(&mut self, to: LoopState) {
        if self.ongoing_loops.is_empty() {
            return;
        }
        self.pop_loop();
        self.push_loop(to);
    }

    fn is_loop_at(&self, state: LoopState) -> bool {
        let last_loop = *self.get_last_loop().unwrap();
        last_loop == state
    }

    fn set_loop_state(&mut self, to: LoopState, from: Option<LoopState>) -> bool {
        if from.is_none() {
            return if !self.is_loops_empty() {
                self.set_last_loop(to);
                true
            } else {
                false
            };
        } else if self.is_loop_at(from.unwrap()) {
            self.set_last_loop(to);
        }
        true
    }
}

pub struct Interpreter {
    env: EnvId,
    root_env: EnvId,
    env_man: Environment,
    arena: Arc<AstArena>,
    loop_stack: LoopStack,
    resolver: Arc<Resolver>,
}

impl Interpreter {
    pub fn new(
        env_man: Environment,
        env: EnvId,
        arena: Arc<AstArena>,
        resolver: Arc<Resolver>,
    ) -> Interpreter {
        Interpreter {
            env_man,
            env: env,
            root_env: env,
            arena: Arc::clone(&arena),
            loop_stack: LoopStack::new(),
            resolver,
        }
    }

    fn eval_unary(&mut self, operation: &Operation, eidx: ExprId) -> Result<RV, HaltReason> {
        if *operation == Operation::Subtract {
            if let Some(num) = self.visit_expr(eidx)?.coerce_to_number() {
                return Ok(RV::Num(-num));
            }
            Ok(RV::NaN)
        } else {
            Ok(RV::Bool(!self.visit_expr(eidx)?.is_truthy()))
        }
    }

    fn eval_binary(
        &mut self,
        lidx: ExprId,
        ridx: ExprId,
        operation: Operation,
    ) -> Result<RV, HaltReason> {
        let left_eval = self.visit_expr(lidx)?;
        let right_eval = self.visit_expr(ridx)?;

        Ok(eval_binary(left_eval, right_eval, operation))
    }

    fn look_up_variable(&self, name: &str, eid: ExprId) -> Result<RV, HaltReason> {
        let distance = self.resolver.get_distance(eid);
        if let Some(unwrapped) = distance {
            self.env_man.read_at(self.env, unwrapped, name)
        } else {
            self.env_man.read(self.root_env, name)
        }
    }

    pub fn user_fn_call(
        &mut self,
        statements: &Vec<StmtId>,
        closure: EnvId,
        parameters: &Vec<String>,
        arguments: &[RV],
    ) -> Result<RV, HaltReason> {
        let fn_env = self.env_man.push(Some(closure));

        for (i, param) in parameters.iter().enumerate() {
            // TODO: Remove clone here
            self.env_man
                .declare(fn_env, param.to_string(), arguments.get(i).unwrap().clone());
        }

        self.execute_block(statements, Some(fn_env))
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<StmtId>,
        env_opt: Option<EnvId>,
    ) -> Result<RV, HaltReason> {
        let mut env_tmp: Option<EnvId> = None;

        if let Some(env_opt_unwrapped) = env_opt {
            env_tmp = Some(self.env);
            self.env = env_opt_unwrapped;
        } else {
            self.env = self.env_man.push(Some(self.env));
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
            self.env = self.env_man.pop(self.env);
        }
        ret
    }

    pub fn literal_to_rv(&mut self, literal: &Literal) -> RV {
        match literal {
            Literal::Str(s) => RV::Str(Arc::clone(s)),
            Literal::Num(n) => RV::Num(*n),
            Literal::Bool(b) => RV::Bool(*b),
            Literal::Undefined => RV::Undefined,
            Literal::NaN => RV::NaN,
            Literal::Null => RV::Null,
            Literal::Object(map) => {
                let mut new_map = FxHashMap::default();
                for (k, v) in map.iter() {
                    new_map.insert(k.clone(), self.visit_expr(*v).unwrap());
                }
                RV::Object(alloc_shared(new_map))
            }
            Literal::Array(arr) => {
                let collected = arr.iter().map(|x| self.visit_expr(*x).unwrap()).collect();
                RV::Array(alloc_shared(collected))
            }
        }
    }
}

impl VisitorMut<RV, HaltReason> for Interpreter {
    fn visit_expr(&mut self, eidx: ExprId) -> Result<RV, HaltReason> {
        // TODO: Remove clone here
        let a = Arc::clone(&self.arena);
        let e = a.get_expression(eidx);
        match e {
            Expr::Select { query, span: _ } => Ok(RV::Str(Arc::new(format!("{:?}", query)))),
            Expr::Insert { command, span: _ } => Ok(RV::Str(Arc::new(format!("{:?}", command)))),
            Expr::Update { command, span: _ } => Ok(RV::Str(Arc::new(format!("{:?}", command)))),
            Expr::Delete { command, span: _ } => Ok(RV::Str(Arc::new(format!("{:?}", command)))),
            Expr::Literal {
                value,
                raw: _,
                span: _,
            } => Ok(self.literal_to_rv(&value)),
            Expr::Grouping { expr, span: _ } => self.visit_expr(*expr),
            Expr::Unary {
                operation,
                expr,
                span: _,
            } => self.eval_unary(operation, *expr),
            Expr::Binary {
                operation,
                left,
                right,
                span: _,
            } => self.eval_binary(*left, *right, *operation),
            Expr::Variable { name, span: _ } => self.look_up_variable(&name.name, eidx),
            Expr::Assignment { dst, expr, span: _ } => {
                let distance = self.resolver.get_distance(eidx);
                let evaluated = self.visit_expr(*expr)?;
                let result = if let Some(distance_unv) = distance {
                    self.env_man
                        .assign_at(self.env, distance_unv, &dst.name, evaluated.clone())
                } else {
                    self.env_man
                        .assign(self.env, dst.name.clone(), evaluated.clone())
                };
                if result.is_err() {
                    return Err(result.err().unwrap());
                }
                Ok(evaluated)
            }
            Expr::Logical {
                left,
                operation,
                right,
                span: _,
            } => {
                let is_true = self.visit_expr(*left)?.is_truthy();

                if (*operation == Operation::Or && is_true)
                    || (*operation == Operation::And && !is_true)
                {
                    return Ok(RV::Bool(is_true));
                }

                Ok(RV::Bool(self.visit_expr(*right)?.is_truthy()))
            }
            Expr::Call { callee, args, span } => {
                let eval = self.visit_expr(*callee)?;

                if let Callable(arity, callable) = eval {
                    if arity.is_some() && arity.unwrap() != args.len() {
                        return Err(HaltReason::Error(InterpretError::ArityMismatch {
                            span: *span,
                            expected: arity.unwrap(),
                            found: args.len(),
                        }));
                    }

                    let mut args_evaluated: Vec<RV> = vec![];

                    for arg in args.iter() {
                        args_evaluated.push(self.visit_expr(*arg)?);
                    }
                    self.loop_stack.push_fn();

                    let val = callable.call(self, args_evaluated.as_slice());

                    self.loop_stack.pop_fn();
                    match val {
                        Err(HaltReason::Return(ret_val)) => Ok(ret_val),
                        Ok(unpacked_val) => Ok(unpacked_val),
                        other_err @ Err(_) => other_err,
                    }
                } else {
                    Err(HaltReason::Error(InterpretError::NotCallable {
                        span: self.arena.get_expression(*callee).get_span(),
                    }))
                }
            }
            Expr::Function {
                name,
                parameters,
                body,
                span: _,
            } => {
                let fn_name = if name.is_some() {
                    &name.as_ref().unwrap().name
                } else {
                    "<anonymous>"
                };
                let fun = Function::UserDefined {
                    name: fn_name.to_string(),
                    body: Arc::clone(body),
                    parameters: parameters.iter().map(|x| x.name.to_string()).collect(),
                    closure: self.env.clone(),
                };

                let callable = Callable(Some(parameters.len()), fun.into());

                if name.is_some() {
                    // TODO(vck): Callable shouldn't be cloned here
                    self.env_man.declare(
                        self.env,
                        name.as_ref().unwrap().name.to_string(),
                        callable.clone(),
                    );
                }

                Ok(callable)
            }
            Expr::Get { object, name, span } => {
                let object_eval = self.visit_expr(*object)?;
                if let RV::Object(map) = object_eval {
                    let cloned = map.clone();
                    let borrowed = cloned.read().unwrap();
                    let v = borrowed.get(&name.name.clone());
                    if v.is_some() {
                        return Ok(v.unwrap().clone());
                    }
                    Err(HaltReason::Error(InterpretError::PropertyNotFound {
                        span: *span,
                        property: name.name.to_string(),
                    }))
                } else {
                    Err(HaltReason::Error(InterpretError::Other {
                        message: format!(
                            "Only objects have properties. {:?} is not an object",
                            object_eval
                        ),
                    }))
                }
            }
            Expr::Set {
                object,
                name,
                value,
                span: _,
            } => {
                let object_eval = self.visit_expr(*object)?;
                if let RV::Object(map) = object_eval {
                    let evaluated = self.visit_expr(*value)?;
                    // TODO(vck): Set should really set the value
                    let mut borrowed = map.write().unwrap();
                    borrowed.insert(name.name.to_string(), evaluated.clone());
                    Ok(evaluated)
                } else {
                    Err(HaltReason::Error(InterpretError::Other {
                        message: format!(
                            "Only objects have properties. {:?} is not an object",
                            object_eval
                        ),
                    }))
                }
            }
        }
    }

    fn visit_stmt(&mut self, sidx: StmtId) -> Result<RV, HaltReason> {
        if !self.loop_stack.is_loops_empty()
            && *self.loop_stack.get_last_loop().unwrap() != LoopState::Go
        {
            return Ok(RV::Undefined);
        }
        // TODO: Remove clone here
        let a = Arc::clone(&self.arena);
        let s = a.get_statement(sidx);
        match s {
            Stmt::Program {
                body: stmts,
                span: _,
            } => {
                return self.execute_block(stmts, Some(self.env.clone()));
            }
            Stmt::Expression { expr, span: _ } => {
                return self.visit_expr(*expr);
            }
            Stmt::Declaration { dst, expr, span: _ } => {
                let evaluated = self.visit_expr(*expr)?;
                self.env_man
                    .declare(self.env, dst.name.to_string(), evaluated.clone());
            }
            Stmt::Block {
                body: stmts,
                span: _,
            } => {
                return self.execute_block(stmts, None);
            }
            Stmt::If {
                condition,
                body,
                r#else_body: r#else,
                span: _,
            } => {
                if self.visit_expr(*condition)?.is_truthy() {
                    self.visit_stmt(*body)?;
                } else if let Some(else_stmt) = r#else {
                    self.visit_stmt(*else_stmt)?;
                }
            }
            Stmt::Loop {
                condition,
                body,
                post,
                span: _,
            } => {
                self.loop_stack.push_loop(LoopState::Go);
                while !self.loop_stack.is_loop_at(LoopState::Broken)
                    && (condition.is_none()
                        || self.visit_expr(condition.unwrap())?.is_truthy())
                {
                    self.visit_stmt(*body)?;
                    self.loop_stack
                        .set_loop_state(LoopState::Go, Some(LoopState::Continue));
                    if let Some(post_id) = post {
                        self.visit_stmt(*post_id)?;
                    }
                }
                self.loop_stack.pop_loop();
            }
            Stmt::Break { span } => {
                if !self.loop_stack.set_loop_state(LoopState::Broken, None) {
                    return Err(HaltReason::Error(InterpretError::UnexpectedStatement {
                        span: *span,
                    }));
                }
            }
            Stmt::Continue { span } => {
                if !self.loop_stack.set_loop_state(LoopState::Continue, None) {
                    return Err(HaltReason::Error(InterpretError::UnexpectedStatement {
                        span: *span,
                    }));
                }
            }
            Stmt::Return { span: _, expr } => {
                if expr.is_some() {
                    let ret = self.visit_expr(expr.unwrap())?;
                    return Err(HaltReason::Return(ret));
                }
                return Err(HaltReason::Return(RV::Undefined));
            }
        }
        Ok(RV::Undefined)
    }
}

#[derive(Clone)]
pub struct Output {
    out: Vec<RV>,
}

impl Output {
    pub fn new() -> Output {
        Output { out: Vec::new() }
    }

    pub fn push(&mut self, rv: RV) {
        self.out.push(rv);
    }

    pub fn expect(&mut self, rv: Vec<RV>) {
        assert_eq!(self.out, rv);
    }
}

impl Stateful for Output {
    fn call(&mut self, _interpreter: &mut Interpreter, rv: &[RV]) -> Result<RV, HaltReason> {
        for item in rv {
            self.push(item.clone());
        }
        Ok(RV::Undefined)
    }
}

pub mod test_helpers {
    use crate::runtime::types::RV;
    use crate::runtime::{Runtime, RuntimeMode};
    use crate::util::{alloc_shared, Shared};

    use super::Output;

    pub fn get_runtime() -> (Shared<Output>, Runtime) {
        let out = alloc_shared(Output::new());

        (
            out.clone(),
            Runtime {
                out: Some(out),
                mode: RuntimeMode::File,
            },
        )
    }

    pub fn exec_assert(code: &str, output: Vec<RV>) -> () {
        let (out, mut runtime) = get_runtime();
        runtime.interpret(code);
        out.write().unwrap().expect(output);
    }
}

#[cfg(test)]
mod test {
    use crate::runtime::types::RV;

    use super::test_helpers::get_runtime;

    #[test]
    fn test_unary_evaluation() {
        let code = "
            TestUtils.out(-2);
            TestUtils.out(-(-2));
            TestUtils.out(!3);
            TestUtils.out(!!3);
            TestUtils.out(!!!3);
        ";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.write().unwrap().expect(vec![
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
            TestUtils.out(5-(-2));
            TestUtils.out((5 + 2) * 4);
            TestUtils.out(5 + 2 * 4);
            TestUtils.out((13 + 4) * (7 + 3));
            TestUtils.out(-5-2);
        ";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.write().unwrap().expect(vec![
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
            TestUtils.out(5 && 1);
            TestUtils.out(5 || 1);
            TestUtils.out(5 && 0);
            TestUtils.out(5 || 0);
            TestUtils.out(!(5 || 0));
            TestUtils.out(!(5 || 0) || 1);
            TestUtils.out(!(5 || 0) || (1 && 0));
        ";
        let (out, mut runtime) = get_runtime();
        runtime.interpret(&code);
        out.write().unwrap().expect(vec![
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
