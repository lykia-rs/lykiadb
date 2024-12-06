use super::environment::EnvId;
use super::RV;
use crate::{
    engine::interpreter::{HaltReason, Interpreter},
    util::Shared,
};
use lykiadb_lang::ast::stmt::Stmt;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum CallableKind {
    Generic,
    Aggregator,
}

#[derive(Clone, Debug)]
pub struct Callable {
    pub arity: Option<usize>,
    pub kind: CallableKind,
    pub function: Arc<Function>,
}

impl Callable {
    pub fn new(arity: Option<usize>, call_type: CallableKind, function: Function) -> Self {
        Callable {
            arity,
            kind: call_type,
            function: Arc::new(function),
        }
    }

    pub fn call(&self, interpreter: &mut Interpreter, arguments: &[RV]) -> Result<RV, HaltReason> {
        match self.function.as_ref() {
            Function::Stateful(stateful) => stateful.write().unwrap().call(interpreter, arguments),
            Function::Lambda { function } => function(interpreter, arguments),
            Function::UserDefined {
                parameters,
                closure,
                body,
                ..
            } => interpreter.user_fn_call(body, *closure, parameters, arguments),
        }
    }
}

pub trait Stateful {
    fn call(&mut self, interpreter: &mut Interpreter, rv: &[RV]) -> Result<RV, HaltReason>;
}

#[derive(Clone)]
pub enum Function {
    Lambda {
        function: fn(&mut Interpreter, &[RV]) -> Result<RV, HaltReason>,
    },
    Stateful(Shared<dyn Stateful + Send + Sync>),
    UserDefined {
        name: String,
        parameters: Vec<String>,
        closure: EnvId,
        body: Arc<Vec<Stmt>>,
    },
}

impl Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Stateful(_) | Function::Lambda { .. } => write!(f, "<native_fn>"),
            Function::UserDefined { name, .. } => write!(f, "{}", name),
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
