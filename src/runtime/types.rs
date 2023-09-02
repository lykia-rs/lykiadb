use serde::{Deserialize, Serialize};
use std::rc::Rc;
use rustc_hash::FxHashMap;
use std::fmt::{Debug, Formatter, Display};
use std::process::exit;
use crate::runtime::environment::{Environment, Shared};
use crate::runtime::interpreter::{HaltReason, Interpreter};
use crate::lang::ast::Stmt;

#[derive(Clone)]
pub enum Function {
    Native {
        function: fn(&mut Interpreter, &[RV]) -> Result<RV, HaltReason>
    },
    UserDefined {
        name: String,
        parameters: Vec<String>, 
        closure: Shared<Environment>,
        body: Rc<Vec<Stmt>>
    },
}

impl Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Native { function: _ } => write!(f, "<native_fn>"),
            Function::UserDefined { name, parameters: _, closure: _, body: _ } => write!(f, "{}", name),
            _ => exit(1)
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Native { function: _ }, Function::Native { function: _ }) => false,
            (a @ Function::UserDefined { name: _, parameters: _, closure: _, body: _ }, 
             b @ Function::UserDefined { name: _, parameters: _, closure: _, body: _ }) => {
                a == b
            }
            _ => false
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
            Function::Native { function } => function(interpreter, arguments),
            Function::UserDefined { name: _, parameters, closure, body }  => {
                let fn_env = Environment::new(Some(Rc::clone(&closure)));

                for (i, param) in parameters.iter().enumerate() {
                    // TODO: Remove clone here
                    fn_env.borrow_mut().declare(param.to_string(), arguments.get(i).unwrap().clone());
                }

                interpreter.user_fn_call(&body, fn_env)
            }
            _ => exit(1)
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum RV {
    Str(Rc<String>),
    Num(f64),
    Bool(bool),
    Object(FxHashMap<String, RV>),
    Array(Vec<RV>),
    Callable(Option<usize>, Rc<Function>),
    Undefined,
    NaN,
    Null,
}

impl Eq for RV {}

impl<'de> Deserialize<'de> for RV {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(s) => Ok(RV::Str(Rc::new(s))),
            serde_json::Value::Number(n) => Ok(RV::Num(n.as_f64().unwrap())),
            serde_json::Value::Bool(b) => Ok(RV::Bool(b)),
            serde_json::Value::Null => Ok(RV::Null),
            _ => Ok(RV::Undefined)
        }
    }
}

impl Serialize for RV {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
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
