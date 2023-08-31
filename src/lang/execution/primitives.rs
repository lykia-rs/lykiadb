use std::fmt::{Debug, Formatter};
use std::process::exit;
use std::rc::Rc;
use crate::lang::execution::environment::{Environment, Shared};
use crate::lang::execution::interpreter::Interpreter;
use crate::lang::parsing::ast::Stmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum RV {
    Str(Rc<String>),
    Num(f64),
    Bool(bool),
    Undefined,
    NaN,
    Nil,
    Callable(Option<usize>, Rc<Function>)
}

impl<'de> Deserialize<'de> for RV {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(s) => Ok(RV::Str(Rc::new(s))),
            serde_json::Value::Number(n) => Ok(RV::Num(n.as_f64().unwrap())),
            serde_json::Value::Bool(b) => Ok(RV::Bool(b)),
            serde_json::Value::Null => Ok(RV::Nil),
            _ => Ok(RV::Undefined)
        }
    }
}

impl<'se> Serialize for RV {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        match self {
            RV::Str(s) => serializer.serialize_str(s),
            RV::Num(n) => serializer.serialize_f64(*n),
            RV::Bool(b) => serializer.serialize_bool(*b),
            RV::Undefined => serializer.serialize_none(),
            RV::NaN => serializer.serialize_none(),
            RV::Nil => serializer.serialize_none(),
            RV::Callable(_, _) => serializer.serialize_none()
        }
    }
}

#[derive(Debug)]
pub enum HaltReason {
    Error(String),
    Return(RV),
}

pub fn runtime_err(msg: &str, line: u32) -> HaltReason {
    HaltReason::Error(format!("{} at line {}", msg, line + 1))
}

pub enum Function {
    Native(fn(&mut Interpreter, &[RV]) -> Result<RV, HaltReason>),
    UserDefined(String, Rc<Vec<Stmt>>, Vec<String>, Shared<Environment>),
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Native(_) => write!(f, "<native_fn>"),
            Function::UserDefined(name, _, _, _) => write!(f, "{}", name),
            _ => exit(1)
        }
    }
}

impl Function {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: &[RV]) -> Result<RV, HaltReason> {
        match self {
            Function::Native(function) => function(interpreter, arguments),
            Function::UserDefined(_name, body, parameters, closure) => {
                let fn_env = Environment::new(Some(Rc::clone(closure)));

                for (i, param) in parameters.iter().enumerate() {
                    fn_env.borrow_mut().declare(param.to_string(), arguments.get(i).unwrap().clone());
                }

                interpreter.user_fn_call(&body, fn_env).map(|_| RV::Undefined)
            }
            _ => exit(1)
        }
    }
}
