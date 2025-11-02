use super::error::ExecutionError;
use super::stdlib::stdlib;
use lykiadb_common::error::StandardError;
use lykiadb_lang::ast::expr::{Expr, Operation, RangeKind};
use lykiadb_lang::ast::stmt::Stmt;
use lykiadb_lang::ast::visitor::VisitorMut;
use lykiadb_lang::ast::{Literal, Span, Spanned};
use lykiadb_lang::parser::program::Program;
use lykiadb_lang::types::Datatype;
use lykiadb_lang::LangError;
use pretty_assertions::assert_eq;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::global::GLOBAL_INTERNER;
use interb::Symbol;
use crate::plan::planner::Planner;
use crate::exec::PlanExecutor;
use crate::util::Shared;
use crate::value::callable::{CallableKind, Function, RVCallable, Stateful};
use crate::value::environment::EnvironmentFrame;
use crate::value::iterator::IterationEnvironment;
use crate::value::{RV, eval::eval_binary};
use crate::value::{array::RVArray, object::RVObject};

use std::fmt::Display;
use std::sync::Arc;
use std::vec;

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

#[derive(Clone)]
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
        self.ongoing_loops.last()
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

#[derive(Clone)]
pub struct Interpreter {
    env: Arc<EnvironmentFrame>,
    root_env: Arc<EnvironmentFrame>,
    iter_env: Option<IterationEnvironment>,
    //
    program: Option<Arc<Program>>,
    output: Option<Shared<Output>>,
    //
    loop_stack: LoopStack,
}

impl Interpreter {
    pub fn new(out: Option<Shared<Output>>, with_stdlib: bool) -> Interpreter {
        let root_env = Arc::new(EnvironmentFrame::new(None));
        if with_stdlib {
            let native_fns = stdlib(out.clone());

            for (name, value) in native_fns {
                root_env.define(GLOBAL_INTERNER.intern(&name), value);
            }
        }
        Interpreter {
            env: root_env.clone(),
            root_env,
            loop_stack: LoopStack::new(),
            program: None,
            output: out,
            iter_env: None,
        }
    }

    pub fn eval(&mut self, e: &Expr) -> Result<RV, HaltReason> {
        self.visit_expr(e)
    }

    pub fn interpret(&mut self, program: Arc<Program>) -> Result<RV, ExecutionError> {
        self.program = Some(program.clone());
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
            Ok(RV::Undefined)
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

    pub fn set_iter_env(&mut self, env: Option<IterationEnvironment>) {
        self.iter_env = env;
    }

    pub fn clear_iter_env(&mut self) {
        self.iter_env = None;
    }

    fn intern_string(&self, string: &str) -> Symbol {
        GLOBAL_INTERNER.intern(string)
    }

    fn look_up_variable(&mut self, name: &str, expr: &Expr) -> Result<RV, HaltReason> {
        if self.iter_env.is_some() {
            let iter_env = self.iter_env.as_ref().unwrap();
            if let Some(val) = iter_env.get(&self.intern_string(name)) {
                return Ok(val.clone());
            }
        }

        let distance = self
            .program
            .as_ref()
            .and_then(|x| x.get_distance(expr));
        if let Some(unwrapped) = distance {
            EnvironmentFrame::read_at(
                &self.env,
                unwrapped,
                name,
                &self.intern_string(name),
            )
        } else {
            self.root_env.read(name, &self.intern_string(name))
        }
    }

    pub fn user_fn_call(
        &mut self,
        statements: &Vec<Stmt>,
        closure: Arc<EnvironmentFrame>,
        parameters: &[Symbol],
        arguments: &[RV],
    ) -> Result<RV, HaltReason> {
        let fn_env = EnvironmentFrame::new(Some(Arc::clone(&closure)));

        for (i, param) in parameters.iter().enumerate() {
            // TODO: Remove clone here
            fn_env.define(*param, arguments.get(i).unwrap().clone());
        }

        self.execute_block(statements, Arc::new(fn_env))
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        env_opt: Arc<EnvironmentFrame>,
    ) -> Result<RV, HaltReason> {
        let previous = std::mem::replace(&mut self.env, env_opt);

        let mut ret = Ok(RV::Undefined);

        for statement in statements {
            ret = self.visit_stmt(statement);
            if ret.is_err() {
                break;
            }
        }

        self.env = previous;

        ret
    }

    fn literal_to_rv(&mut self, literal: &Literal) -> RV {
        match literal {
            Literal::Str(s) => RV::Str(Arc::clone(s)),
            Literal::Num(n) => RV::Num(*n),
            Literal::Bool(b) => RV::Bool(*b),
            Literal::Undefined => RV::Undefined,
            Literal::Object(map) => {
                let mut new_map = FxHashMap::default();
                for (k, v) in map.iter() {
                    new_map.insert(k.clone(), self.visit_expr(v).unwrap());
                }
                RV::Object(RVObject::from_map(new_map))
            }
            Literal::Array(arr) => {
                let collected = arr.iter().map(|x| self.visit_expr(x).unwrap()).collect();
                RV::Array(RVArray::from_vec(collected))
            }
        }
    }
}

impl VisitorMut<RV, HaltReason> for Interpreter {
    fn visit_expr(&mut self, e: &Expr) -> Result<RV, HaltReason> {
        match e {
            Expr::Literal { value, .. } => Ok(self.literal_to_rv(value)),
            Expr::Variable { name, .. } => self.look_up_variable(&name.name, e),
            Expr::Unary {
                operation, expr, ..
            } => self.eval_unary(operation, expr),
            Expr::Binary {
                operation,
                left,
                right,
                ..
            } => self.eval_binary(left, right, *operation),
            Expr::Grouping { expr, .. } => self.visit_expr(expr),
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
            Expr::Assignment { dst, expr, .. } => {
                let distance = self.program.as_ref().unwrap().get_distance(e);
                let evaluated = self.visit_expr(expr)?;
                let dst_symbol = self.intern_string(&dst.name);
                let result = if let Some(distance_unv) = distance {
                    EnvironmentFrame::assign_at(
                        &self.env,
                        distance_unv,
                        &dst.name,
                        dst_symbol,
                        evaluated.clone(),
                    )
                } else {
                    self.root_env.assign(
                        &dst.name,
                        dst_symbol,
                        evaluated.clone(),
                    )
                };
                if result.is_err() {
                    return Err(result.err().unwrap());
                }
                Ok(evaluated)
            }
            Expr::Call { callee, args, .. } => {
                let eval = self.visit_expr(callee)?;

                if let RV::Callable(callable) = eval {
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

                let param_identifiers = parameters
                    .iter()
                    .map(|(x, _)| self.intern_string(&x.name))
                    .collect();

                let fun = Function::UserDefined {
                    name: self.intern_string(fn_name),
                    body: Arc::clone(body),
                    parameters: param_identifiers,
                    closure: self.env.clone(),
                };

                // TODO(vck): Type evaluation should be moved to a pre-execution phase
                let callable = RV::Callable(RVCallable::new(
                    fun,
                    Datatype::Unit,
                    Datatype::Unit,
                    CallableKind::Generic,
                ));

                if name.is_some() {
                    // TODO(vck): Callable shouldn't be cloned here
                    self.env.define(
                        self.intern_string(&name.as_ref().unwrap().name),
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
                                "Range can only be created with numbers. {lower_eval:?} {upper_eval:?} {subject_eval:?}"
                            ),
                        }
                        .into(),
                    ))
                }
            }
            Expr::FieldPath { head, tail, span, id } => {
                let root = self.look_up_variable(&head.name, e);

                if tail.is_empty() {
                    return root;
                }

                let mut current = root?;

                for field in tail {
                    if let RV::Object(map) = current {
                        let v = map.get(&field.name);
                        if let Some(v) = v {
                            current = v;
                        } else {
                            return Err(HaltReason::Error(
                                InterpretError::PropertyNotFound {
                                    span: *span,
                                    property: field.name.to_string(),
                                }
                                .into(),
                            ));
                        }
                    } else {
                        return Err(HaltReason::Error(
                            InterpretError::Other {
                                message: format!(
                                    "Only objects have properties. {current:?} is not an object"
                                ),
                            }
                            .into(),
                        ));
                    }
                };

                Ok(current)
            },
            Expr::Get {
                object, name, span, ..
            } => {
                let object_eval = self.visit_expr(object)?;
                if let RV::Object(map) = object_eval {
                    let v = map.get(&name.name.clone());
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
                                "Only objects have properties. {object_eval:?} is not an object"
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
                if let RV::Object(mut map) = object_eval {
                    let evaluated = self.visit_expr(value)?;
                    map.insert(name.name.to_string(), evaluated.clone());
                    Ok(evaluated)
                } else {
                    Err(HaltReason::Error(
                        InterpretError::Other {
                            message: format!(
                                "Only objects have properties. {object_eval:?} is not an object"
                            ),
                        }
                        .into(),
                    ))
                }
            }
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
                let result = PlanExecutor::new(self).execute_plan(plan);
                if let Err(e) = result {
                    return Err(HaltReason::Error(e));
                }
                let cursor = result.ok().unwrap();
                let intermediate = cursor.map(|env| {
                    let mut row: FxHashMap<String, RV> = FxHashMap::default();
                    for (key, value) in env.inner.iter() {
                        row.insert(GLOBAL_INTERNER.resolve(*key).unwrap().to_string(), value.clone());
                    }
                    RV::Object(RVObject::from_map(row))
                }).collect::<Vec<RV>>();
                println!("Intermediate execution result: {:?}", intermediate);
                Ok(RV::Undefined)
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
                return self.execute_block(stmts, self.env.clone());
            }
            Stmt::Expression { expr, .. } => {
                return self.visit_expr(expr);
            }
            Stmt::Declaration { dst, expr, .. } => {
                let evaluated = self.visit_expr(expr)?;
                self.env
                    .define(self.intern_string(&dst.name), evaluated);
            }
            Stmt::Block { body: stmts, .. } => {
                return self.execute_block(
                    stmts,
                    Arc::new(EnvironmentFrame::new(Some(Arc::clone(&self.env)))),
                );
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregation {
    pub name: String,
    pub args: Vec<Expr>,
}

impl Display for Aggregation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.args
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
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

    pub fn clear(&mut self) {
        self.out.clear();
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

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum InterpretError {
    #[error("Expression is not callable at {span:?}")]
    NotCallable { span: Span },
    #[error("Unexpected statement at {span:?}")]
    UnexpectedStatement { span: Span },
    #[error("Property '{property}' not found at {span:?}")]
    PropertyNotFound { span: Span, property: String },
    #[error("{message}")]
    Other { message: String }, // TODO(vck): Refactor this
}

impl From<InterpretError> for StandardError {
    fn from(value: InterpretError) -> Self {
        let (hint, sp) = match &value {
            InterpretError::NotCallable { span } => (
                "Ensure the expression evaluates to a callable function",
                *span,
            ),
            InterpretError::UnexpectedStatement { span } => (
                "Check if the statement is used in the correct context",
                *span,
            ),
            InterpretError::PropertyNotFound { span, .. } => {
                ("Verify the property name exists on the object", *span)
            }
            InterpretError::Other { .. } => (
                "Review the error details for specific guidance",
                Span::default(),
            ),
        };

        StandardError::new(&value.to_string(), hint, Some(sp.into()))
    }
}

impl From<LangError> for ExecutionError {
    fn from(err: LangError) -> Self {
        ExecutionError::Lang(err)
    }
}

impl From<InterpretError> for ExecutionError {
    fn from(err: InterpretError) -> Self {
        ExecutionError::Interpret(err)
    }
}
