use std::sync::Arc;
use std::fmt::{Debug, Display, Formatter};
use lykiadb_lang::ast::stmt::Stmt;
use crate::{engine::interpreter::{HaltReason, Interpreter}, util::Shared};
use super::environment::EnvId;
use super::types::RV;

#[derive(Debug, Clone)]
pub enum CallableType {
    Any,
    Aggregator,
}

#[derive(Clone, Debug)]
pub struct Callable {
    pub arity: Option<usize>,
    pub call_type: CallableType,
    pub function: Arc<Function>,
}

impl Callable {
    pub fn new(arity: Option<usize>, call_type: CallableType, function: Function) -> Self {
        Callable {
            arity,
            call_type,
            function: Arc::new(function),
        }
    }

    pub fn call(&self, interpreter: &mut Interpreter, arguments: &[RV]) -> Result<RV, HaltReason> {
        match self.function.as_ref() {
            Function::Stateful(stateful) => stateful.write().unwrap().call(interpreter, arguments),
            Function::Lambda { function } => function(interpreter, arguments),
            Function::UserDefined {
                name: _,
                parameters,
                closure,
                body,
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
            Function::Stateful(_) | Function::Lambda { function: _ } => write!(f, "<native_fn>"),
            Function::UserDefined {
                name,
                parameters: _,
                closure: _,
                body: _,
            } => write!(f, "{}", name),
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
