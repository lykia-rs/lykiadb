use lykiadb_lang::ast::expr::{Expr, Operation, RangeKind};
use lykiadb_lang::ast::stmt::Stmt;
use lykiadb_lang::ast::visitor::VisitorMut;
use lykiadb_lang::parser::program::Program;
use lykiadb_lang::parser::resolver::Resolver;
use lykiadb_lang::parser::Parser;
use lykiadb_lang::tokenizer::scanner::Scanner;
use lykiadb_lang::Span;
use lykiadb_lang::Spanned;
use lykiadb_lang::{Literal, Locals, Scopes};
use pretty_assertions::assert_eq;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use super::error::ExecutionError;
use super::stdlib::stdlib;

use crate::plan::planner::Planner;
use crate::util::{alloc_shared, Shared};
use crate::value::callable::{Callable, CallableKind, Function, Stateful};
use crate::value::environment::{EnvId, Environment};
use crate::value::{eval::eval_binary, RV};

use std::sync::Arc;
use std::vec;

pub struct SourceProcessor {
    scopes: Scopes,
    locals: Locals,
}

impl Default for SourceProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceProcessor {
    pub fn new() -> SourceProcessor {
        SourceProcessor {
            scopes: vec![],
            locals: FxHashMap::default(),
        }
    }

    pub fn process(&mut self, source: &str) -> Result<Program, ExecutionError> {
        let tokens = Scanner::scan(source)?;
        let mut program = Parser::parse(&tokens)?;
        let mut resolver = Resolver::new(self.scopes.clone(), &program, Some(self.locals.clone()));
        let (scopes, locals) = resolver.resolve().unwrap();

        self.scopes = scopes;
        self.locals.clone_from(&locals);
        program.set_locals(self.locals.clone());

        Ok(program)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
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
    Other {
        message: String,
    }, // TODO(vck): Refactor this
}

impl From<InterpretError> for ExecutionError {
    fn from(err: InterpretError) -> Self {
        ExecutionError::Interpret(err)
    }
}

#[derive(Debug)]
pub enum HaltReason {
    Error(ExecutionError),
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
    env_man: Shared<Environment>,
    current_program: Option<Arc<Program>>,
    //
    loop_stack: LoopStack,
    source_processor: SourceProcessor,
    output: Option<Shared<Output>>,
}

impl Interpreter {
    pub fn new(out: Option<Shared<Output>>, with_stdlib: bool) -> Interpreter {
        let mut env_man = Environment::new();
        if with_stdlib {
            let native_fns = stdlib(out.clone());
            let env = env_man.top();

            for (name, value) in native_fns {
                env_man.declare(env, name.to_string(), value);
            }
        }
        let env = EnvId(0);
        Interpreter {
            env_man: alloc_shared(env_man),
            env,
            root_env: env,
            loop_stack: LoopStack::new(),
            source_processor: SourceProcessor::new(),
            current_program: None,
            output: out,
        }
    }

    pub fn eval(&mut self, e: &Expr) -> Result<RV, HaltReason> {
        self.visit_expr(e)
    }

    pub fn interpret(&mut self, source: &str) -> Result<RV, ExecutionError> {
        let program = Arc::from(self.source_processor.process(source)?);
        self.current_program = Some(program.clone());
        let out = self.visit_stmt(&program.get_root());
        if let Ok(val) = out {
            Ok(val)
        } else {
            let err = out.err().unwrap();
            match err {
                HaltReason::Return(rv) => Ok(rv),
                HaltReason::Error(interpret_err) => Err(interpret_err),
            }
        }
    }

    fn eval_unary(&mut self, operation: &Operation, expr: &Expr) -> Result<RV, HaltReason> {
        if *operation == Operation::Subtract {
            if let Some(num) = self.visit_expr(expr)?.as_number() {
                return Ok(RV::Num(-num));
            }
            Ok(RV::NaN)
        } else {
            Ok(RV::Bool(!self.visit_expr(expr)?.as_bool()))
        }
    }

    fn eval_binary(
        &mut self,
        lexpr: &Expr,
        rexpr: &Expr,
        operation: Operation,
    ) -> Result<RV, HaltReason> {
        let left_eval = self.visit_expr(lexpr)?;
        let right_eval = self.visit_expr(rexpr)?;

        Ok(eval_binary(left_eval, right_eval, operation))
    }

    fn look_up_variable(&self, name: &str, expr: &Expr) -> Result<RV, HaltReason> {
        let distance = self.current_program.clone().unwrap().get_distance(expr);
        if let Some(unwrapped) = distance {
            self.env_man
                .read()
                .unwrap()
                .read_at(self.env, unwrapped, name)
        } else {
            self.env_man.read().unwrap().read(self.root_env, name)
        }
    }

    pub fn user_fn_call(
        &mut self,
        statements: &Vec<Stmt>,
        closure: EnvId,
        parameters: &[String],
        arguments: &[RV],
    ) -> Result<RV, HaltReason> {
        let fn_env = self.env_man.write().unwrap().push(Some(closure));

        for (i, param) in parameters.iter().enumerate() {
            // TODO: Remove clone here
            self.env_man.write().unwrap().declare(
                fn_env,
                param.to_string(),
                arguments.get(i).unwrap().clone(),
            );
        }

        self.execute_block(statements, Some(fn_env))
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        env_opt: Option<EnvId>,
    ) -> Result<RV, HaltReason> {
        let mut env_tmp: Option<EnvId> = None;

        if let Some(env_opt_unwrapped) = env_opt {
            env_tmp = Some(self.env);
            self.env = env_opt_unwrapped;
        } else {
            self.env = self.env_man.write().unwrap().push(Some(self.env));
        }
        let mut ret = Ok(RV::Undefined);

        for statement in statements {
            ret = self.visit_stmt(statement);
            if ret.is_err() {
                break;
            }
        }
        if let Some(env_tmp_unwrapped) = env_tmp {
            self.env = env_tmp_unwrapped;
        } else {
            self.env = self.env_man.write().unwrap().remove(self.env);
        }
        ret
    }

    fn literal_to_rv(&mut self, literal: &Literal) -> RV {
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
                    new_map.insert(k.clone(), self.visit_expr(v).unwrap());
                }
                RV::Object(alloc_shared(new_map))
            }
            Literal::Array(arr) => {
                let collected = arr.iter().map(|x| self.visit_expr(x).unwrap()).collect();
                RV::Array(alloc_shared(collected))
            }
        }
    }
}

impl VisitorMut<RV, HaltReason> for Interpreter {
    fn visit_expr(&mut self, e: &Expr) -> Result<RV, HaltReason> {
        match e {
            Expr::Select { .. }
            | Expr::Insert { .. }
            | Expr::Update { .. }
            | Expr::Delete { .. } => {
                let mut planner = Planner::new(self);
                let plan = planner.build(e)?;
                if let Some(out) = &self.output {
                    out.write()
                        .unwrap()
                        .push(RV::Str(Arc::new(plan.to_string().trim().to_string())));
                }
                Ok(RV::Undefined)
            }
            Expr::Literal { value, .. } => Ok(self.literal_to_rv(value)),
            Expr::Grouping { expr, .. } => self.visit_expr(expr),
            Expr::Unary {
                operation, expr, ..
            } => self.eval_unary(operation, expr),
            Expr::Binary {
                operation,
                left,
                right,
                ..
            } => self.eval_binary(left, right, *operation),
            Expr::Variable { name, .. } => self.look_up_variable(&name.name, e),
            Expr::Assignment { dst, expr, .. } => {
                let distance = self.current_program.clone().unwrap().get_distance(e);
                let evaluated = self.visit_expr(expr)?;
                let result = if let Some(distance_unv) = distance {
                    self.env_man.write().unwrap().assign_at(
                        self.env,
                        distance_unv,
                        &dst.name,
                        evaluated.clone(),
                    )
                } else {
                    self.env_man.write().unwrap().assign(
                        self.env,
                        dst.name.clone(),
                        evaluated.clone(),
                    )
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
                ..
            } => {
                let is_true = self.visit_expr(left)?.as_bool();

                if (*operation == Operation::Or && is_true)
                    || (*operation == Operation::And && !is_true)
                {
                    return Ok(RV::Bool(is_true));
                }

                Ok(RV::Bool(self.visit_expr(right)?.as_bool()))
            }
            Expr::Call {
                callee, args, span, ..
            } => {
                let eval = self.visit_expr(callee)?;

                if let RV::Callable(callable) = eval {
                    if callable.arity.is_some() && callable.arity.unwrap() != args.len() {
                        return Err(HaltReason::Error(
                            InterpretError::ArityMismatch {
                                span: *span,
                                expected: callable.arity.unwrap(),
                                found: args.len(),
                            }
                            .into(),
                        ));
                    }

                    let mut args_evaluated: Vec<RV> = vec![];

                    for arg in args.iter() {
                        args_evaluated.push(self.visit_expr(arg)?);
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
                    Err(HaltReason::Error(
                        InterpretError::NotCallable {
                            span: callee.get_span(),
                        }
                        .into(),
                    ))
                }
            }
            Expr::Function {
                name,
                parameters,
                body,
                ..
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
                    closure: self.env,
                };

                let callable = RV::Callable(Callable::new(
                    Some(parameters.len()),
                    CallableKind::Generic,
                    fun,
                ));

                if name.is_some() {
                    // TODO(vck): Callable shouldn't be cloned here
                    self.env_man.write().unwrap().declare(
                        self.env,
                        name.as_ref().unwrap().name.to_string(),
                        callable.clone(),
                    );
                }

                Ok(callable)
            }
            Expr::Between {
                lower,
                upper,
                subject,
                kind,
                ..
            } => {
                let lower_eval = self.visit_expr(lower)?;
                let upper_eval = self.visit_expr(upper)?;
                let subject_eval = self.visit_expr(subject)?;

                if let (RV::Num(lower_num), RV::Num(upper_num), RV::Num(subject_num)) =
                    (lower_eval.clone(), upper_eval.clone(), subject_eval.clone())
                {
                    let min_num = lower_num.min(upper_num);
                    let max_num = lower_num.max(upper_num);

                    match kind {
                        RangeKind::Between => {
                            Ok(RV::Bool(min_num <= subject_num && subject_num <= max_num))
                        }
                        RangeKind::NotBetween => {
                            Ok(RV::Bool(min_num > subject_num || subject_num > max_num))
                        }
                    }
                } else {
                    Err(HaltReason::Error(
                        InterpretError::Other {
                            message: format!(
                                //TODO: Maybe with dates and strings too?
                                "Range can only be created with numbers. {:?} {:?} {:?}",
                                lower_eval, upper_eval, subject_eval
                            ),
                        }
                        .into(),
                    ))
                }
            }
            Expr::FieldPath { .. } => Err(HaltReason::Error(
                InterpretError::Other {
                    message: "Unexpected field path expression".to_string(),
                }
                .into(),
            )),
            Expr::Get {
                object, name, span, ..
            } => {
                let object_eval = self.visit_expr(object)?;
                if let RV::Object(map) = object_eval {
                    let cloned = map.clone();
                    let borrowed = cloned.read().unwrap();
                    let v = borrowed.get(&name.name.clone());
                    if let Some(v) = v {
                        return Ok(v.clone());
                    }
                    Err(HaltReason::Error(
                        InterpretError::PropertyNotFound {
                            span: *span,
                            property: name.name.to_string(),
                        }
                        .into(),
                    ))
                } else {
                    Err(HaltReason::Error(
                        InterpretError::Other {
                            message: format!(
                                "Only objects have properties. {:?} is not an object",
                                object_eval
                            ),
                        }
                        .into(),
                    ))
                }
            }
            Expr::Set {
                object,
                name,
                value,
                ..
            } => {
                let object_eval = self.visit_expr(object)?;
                if let RV::Object(map) = object_eval {
                    let evaluated = self.visit_expr(value)?;
                    // TODO(vck): Set should really set the value
                    let mut borrowed = map.write().unwrap();
                    borrowed.insert(name.name.to_string(), evaluated.clone());
                    Ok(evaluated)
                } else {
                    Err(HaltReason::Error(
                        InterpretError::Other {
                            message: format!(
                                "Only objects have properties. {:?} is not an object",
                                object_eval
                            ),
                        }
                        .into(),
                    ))
                }
            }
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) -> Result<RV, HaltReason> {
        if !self.loop_stack.is_loops_empty()
            && *self.loop_stack.get_last_loop().unwrap() != LoopState::Go
        {
            return Ok(RV::Undefined);
        }

        match s {
            Stmt::Program { body: stmts, .. } => {
                return self.execute_block(stmts, Some(self.env));
            }
            Stmt::Expression { expr, .. } => {
                return self.visit_expr(expr);
            }
            Stmt::Declaration { dst, expr, .. } => {
                let evaluated = self.visit_expr(expr)?;
                self.env_man.write().unwrap().declare(
                    self.env,
                    dst.name.to_string(),
                    evaluated.clone(),
                );
            }
            Stmt::Block { body: stmts, .. } => {
                return self.execute_block(stmts, None);
            }
            Stmt::If {
                condition,
                body,
                r#else_body: r#else,
                ..
            } => {
                if self.visit_expr(condition)?.as_bool() {
                    self.visit_stmt(body)?;
                } else if let Some(else_stmt) = r#else {
                    self.visit_stmt(else_stmt)?;
                }
            }
            Stmt::Loop {
                condition,
                body,
                post,
                ..
            } => {
                self.loop_stack.push_loop(LoopState::Go);
                while !self.loop_stack.is_loop_at(LoopState::Broken)
                    && (condition.is_none()
                        || self.visit_expr(condition.as_ref().unwrap())?.as_bool())
                {
                    self.visit_stmt(body)?;
                    self.loop_stack
                        .set_loop_state(LoopState::Go, Some(LoopState::Continue));
                    if let Some(post_id) = post {
                        self.visit_stmt(post_id)?;
                    }
                }
                self.loop_stack.pop_loop();
            }
            Stmt::Break { span } => {
                if !self.loop_stack.set_loop_state(LoopState::Broken, None) {
                    return Err(HaltReason::Error(
                        InterpretError::UnexpectedStatement { span: *span }.into(),
                    ));
                }
            }
            Stmt::Continue { span } => {
                if !self.loop_stack.set_loop_state(LoopState::Continue, None) {
                    return Err(HaltReason::Error(
                        InterpretError::UnexpectedStatement { span: *span }.into(),
                    ));
                }
            }
            Stmt::Return { expr, .. } => {
                if expr.is_some() {
                    let ret = self.visit_expr(expr.as_ref().unwrap())?;
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

impl Default for Output {
    fn default() -> Self {
        Self::new()
    }
}

impl Output {
    pub fn new() -> Output {
        Output { out: Vec::new() }
    }

    pub fn push(&mut self, rv: RV) {
        self.out.push(rv);
    }

    pub fn expect(&mut self, rv: Vec<RV>) {
        if rv.len() == 1 {
            if let Some(first) = rv.first() {
                assert_eq!(
                    self.out.first().unwrap_or(&RV::Undefined).to_string(),
                    first.to_string()
                );
            }
        }
        assert_eq!(self.out, rv)
    }
    // TODO(vck): Remove this
    pub fn expect_str(&mut self, rv: Vec<String>) {
        assert_eq!(
            self.out
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            rv
        )
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
