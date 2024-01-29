use crate::lang::ast::stmt::StmtId;
use crate::runtime::interpreter::{HaltReason, Interpreter};
use crate::util::Shared;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use super::environment::EnvId;

pub trait Stateful {
    fn call(&mut self, interpreter: &mut Interpreter, rv: &[RV]) -> Result<RV, HaltReason>;
}

#[derive(Clone)]
pub enum Function {
    Lambda {
        function: fn(&mut Interpreter, &[RV]) -> Result<RV, HaltReason>,
    },
    Stateful(Shared<dyn Stateful>),
    UserDefined {
        name: String,
        parameters: Vec<String>,
        closure: EnvId,
        body: Arc<Vec<StmtId>>,
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

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Lambda { function: _ }, Function::Lambda { function: _ }) => false,
            (
                a @ Function::UserDefined {
                    name: _,
                    parameters: _,
                    closure: _,
                    body: _,
                },
                b @ Function::UserDefined {
                    name: _,
                    parameters: _,
                    closure: _,
                    body: _,
                },
            ) => a == b,
            _ => false,
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

impl Function {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: &[RV]) -> Result<RV, HaltReason> {
        match self {
            Function::Stateful(stateful) => stateful.borrow_mut().call(interpreter, arguments),
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

#[derive(Debug, Clone, PartialEq)]
pub enum RV {
    Str(Arc<String>),
    Num(f64),
    Bool(bool),
    Object(Shared<FxHashMap<String, RV>>),
    Array(Shared<Vec<RV>>),
    Callable(Option<usize>, Arc<Function>),
    Undefined,
    NaN,
    Null,
}

impl Eq for RV {}

impl<'de> Deserialize<'de> for RV {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(s) => Ok(RV::Str(Arc::new(s))),
            serde_json::Value::Number(n) => Ok(RV::Num(n.as_f64().unwrap())),
            serde_json::Value::Bool(b) => Ok(RV::Bool(b)),
            serde_json::Value::Null => Ok(RV::Null),
            _ => Ok(RV::Undefined),
        }
    }
}

impl Serialize for RV {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RV::Str(s) => serializer.serialize_str(s),
            RV::Num(n) => serializer.serialize_f64(*n),
            RV::Bool(b) => serializer.serialize_bool(*b),
            RV::Undefined => serializer.serialize_none(),
            RV::NaN => serializer.serialize_none(),
            RV::Null => serializer.serialize_none(),
            RV::Callable(_, _) => serializer.serialize_none(),
            RV::Array(_) => serializer.serialize_none(),
            RV::Object(_) => serializer.serialize_none(),
        }
    }
}
