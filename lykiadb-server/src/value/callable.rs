use super::RV;
use super::environment::EnvironmentFrame;
use crate::{
    engine::interpreter::{HaltReason, InterpretError, Interpreter}, exec::aggregation::Aggregator, util::Shared
};
use interb::Symbol;
use lykiadb_lang::{
    ast::{Span, stmt::Stmt},
    types::Datatype,
};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct RVCallable {
    pub function: Function,
    pub parameter_types: Datatype,
    pub return_type: Datatype,
}

impl RVCallable {
    pub fn new(
        function: Function,
        input_type: Datatype,
        return_type: Datatype,
    ) -> Self {
        RVCallable {
            function,
            parameter_types: input_type,
            return_type,
        }
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        called_from: &Span,
        arguments: &[RV],
    ) -> Result<RV, HaltReason> {
        match &self.function {
            Function::Stateful(stateful) => stateful.write().unwrap().call(interpreter, arguments),
            Function::Native { function } => function(interpreter, called_from, arguments),
            Function::Agg { .. } => Err(HaltReason::Error(crate::engine::error::ExecutionError::Interpret(InterpretError::InvalidAggregatorCall {
                span: called_from.clone(),
            }))),
            Function::UserDefined {
                parameters,
                closure,
                body,
                ..
            } => interpreter.user_fn_call(body, closure.clone(), parameters, arguments),
        }
    }
}

pub trait Stateful {
    fn call(&mut self, interpreter: &mut Interpreter, rv: &[RV]) -> Result<RV, HaltReason>;
}

pub type AggregatorFactory = fn() -> Box<dyn Aggregator + Send>;

#[derive(Clone)]
pub enum Function {
    Native {
        function: fn(&mut Interpreter, called_from: &Span, &[RV]) -> Result<RV, HaltReason>,
    },
    Stateful(Shared<dyn Stateful + Send + Sync>),
    UserDefined {
        name: Symbol,
        parameters: Vec<Symbol>,
        closure: Arc<EnvironmentFrame>,
        body: Arc<Vec<Stmt>>,
    },
    Agg {
        name: String,
        function: AggregatorFactory,
    },
}

impl Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Stateful(_) | Function::Native { .. } => write!(f, "<native_fn>"),
            Function::UserDefined { .. } => write!(f, "<user_defined_fn>"),
            Function::Agg { .. } => write!(f, "<agg_fn>"),
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}
