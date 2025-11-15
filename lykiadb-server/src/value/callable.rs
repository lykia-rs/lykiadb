use super::RV;
use super::environment::EnvironmentFrame;
use crate::{
    engine::interpreter::{HaltReason, Interpreter},
    util::Shared,
};
use interb::Symbol;
use lykiadb_lang::{
    ast::{Span, stmt::Stmt},
    types::Datatype,
};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum CallableKind {
    Generic,
    Aggregator(String),
}

#[derive(Clone, Debug)]
pub struct RVCallable {
    pub kind: CallableKind,
    pub function: Arc<Function>,
    pub parameter_types: Datatype,
    pub return_type: Datatype,
}

impl RVCallable {
    pub fn new(
        function: Function,
        input_type: Datatype,
        return_type: Datatype,
        call_type: CallableKind,
    ) -> Self {
        RVCallable {
            function: Arc::new(function),
            parameter_types: input_type,
            return_type,
            kind: call_type,
        }
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        called_from: &Span,
        arguments: &[RV],
    ) -> Result<RV, HaltReason> {
        match self.function.as_ref() {
            Function::Stateful(stateful) => stateful.write().unwrap().call(interpreter, arguments),
            Function::Lambda { function } => function(interpreter, called_from, arguments),
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

#[derive(Clone)]
pub enum Function {
    Lambda {
        function: fn(&mut Interpreter, called_from: &Span, &[RV]) -> Result<RV, HaltReason>,
    },
    Stateful(Shared<dyn Stateful + Send + Sync>),
    UserDefined {
        name: Symbol,
        parameters: Vec<Symbol>,
        closure: Arc<EnvironmentFrame>,
        body: Arc<Vec<Stmt>>,
    },
}

impl Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Stateful(_) | Function::Lambda { .. } => write!(f, "<native_fn>"),
            Function::UserDefined { .. } => write!(f, "<user_defined_fn>"),
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
