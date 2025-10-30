use super::environment::EnvironmentFrame;
use crate::{
    engine::interpreter::{HaltReason, Interpreter},
    util::Shared, value::Value,
};
use lykiadb_lang::{ast::stmt::Stmt, types::Datatype};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use string_interner::symbol::SymbolU32;

#[derive(Debug, Clone, PartialEq)]
pub enum CallableKind {
    Generic,
    Aggregator(String),
}

#[derive(Clone, Debug)]
pub struct Callable<V: Value> {
    pub kind: CallableKind,
    pub function: Arc<Function<V>>,
    pub parameter_types: Datatype,
    pub return_type: Datatype,
}

impl<V: Value> Callable<V> {
    pub fn new(
        function: Function<V>,
        input_type: Datatype,
        return_type: Datatype,
        call_type: CallableKind,
    ) -> Self {
        Callable {
            function: Arc::new(function),
            parameter_types: input_type,
            return_type,
            kind: call_type,
        }
    }

    pub fn call(&self, interpreter: &mut Interpreter<V>, arguments: &[V]) -> Result<V, HaltReason<V>> {
        match self.function.as_ref() {
            Function::Stateful(stateful) => stateful.write().unwrap().call(interpreter, arguments),
            Function::Lambda { function } => function(interpreter, arguments),
            Function::UserDefined {
                parameters,
                closure,
                body,
                ..
            } => interpreter.user_fn_call(body, closure.clone(), parameters, arguments),
        }
    }
}

pub trait Stateful<V: Value> {
    fn call(&mut self, interpreter: &mut Interpreter<V>, rv: &[V]) -> Result<V, HaltReason<V>>;
}

#[derive(Clone)]
pub enum Function<V: Value> {
    Lambda {
        function: fn(&mut Interpreter<V>, &[V]) -> Result<V, HaltReason<V>>,
    },
    Stateful(Shared<dyn Stateful<V> + Send + Sync>),
    UserDefined {
        name: SymbolU32,
        parameters: Vec<SymbolU32>,
        closure: Arc<EnvironmentFrame<V>>,
        body: Arc<Vec<Stmt>>,
    },
}

impl<V: Value> Function<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Stateful(_) | Function::Lambda { .. } => write!(f, "<native_fn>"),
            Function::UserDefined { .. } => write!(f, "<user_defined_fn>"),
        }
    }
}

impl<V: Value> Debug for Function<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl<V: Value> Display for Function<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}
