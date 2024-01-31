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
    Stateful(Shared<dyn Stateful + Send + Sync>),
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

#[derive(Debug, Clone)]
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

#[inline(always)]
pub fn is_value_truthy(rv: RV) -> bool {
    match rv {
        RV::Num(value) => !value.is_nan() && value.abs() > 0.0,
        RV::Str(value) => !(value.is_empty() || *value == "0"),
        RV::Bool(value) => value,
        RV::Null | RV::Undefined | RV::NaN => false,
        _ => true,
    }
}

#[inline(always)]
pub fn coerce2number(val: RV) -> Option<f64> {
    match val {
        RV::Num(value) => Some(value),
        RV::Bool(true) => Some(1.0),
        RV::Bool(false) => Some(0.0),
        _ => None,
    }
}

#[inline(always)]
pub fn coerce2bool(val: RV) -> Option<f64> {
    match val {
        RV::Num(value) => Some(value),
        RV::Bool(true) => Some(1.0),
        RV::Bool(false) => Some(0.0),
        _ => None,
    }
}

#[inline(always)]
pub fn cmp_str_num(s: &str, n: f64) -> bool {
    if let Ok(num) = s.parse::<f64>() {
        return num == n;
    }
    false
}

#[inline(always)]
pub fn cmp_any_bool(s: &RV, b: bool) -> bool {
    is_value_truthy(s.clone()) == b
}

impl PartialEq for RV {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RV::Array(_), RV::Array(_)) 
            | (RV::Object(_), RV::Object(_)) => false,
            //
            (RV::Callable(_, _), RV::Callable(_, _)) => false,
            //
            (RV::Null, RV::Null) => true,
            (RV::Undefined, RV::Undefined) => true,
            (RV::NaN, RV::NaN) => true,
            (RV::Null, RV::Undefined) => true,
            (RV::Undefined, RV::Null) => true,
            //
            (RV::NaN, _)
            | (_, RV::NaN) => false,
            (RV::Null, _)
            | (_, RV::Null) => false,
            (RV::Undefined, _)
            | (_, RV::Undefined) => false,
            //
            (RV::Str(a), RV::Str(b)) => a == b,
            (RV::Num(a), RV::Num(b)) => a == b,
            (RV::Bool(a), RV::Bool(b)) => a == b,
            //
            (RV::Str(a), RV::Num(b)) => cmp_str_num(a, *b),
            (RV::Num(a), RV::Str(b)) => cmp_str_num(b, *a),
            //
            (RV::Str(_), RV::Bool(b)) => cmp_any_bool(self, *b),
            (RV::Bool(a), RV::Str(_)) => cmp_any_bool(other, *a),
            //
            (RV::Num(_), RV::Bool(b)) => cmp_any_bool(self, *b),
            (RV::Bool(a), RV::Num(_)) => cmp_any_bool(other, *a),
            //
            _ => false,
        }
    }
}

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

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use rustc_hash::FxHashMap;

    use crate::{
        runtime::types::{coerce2number, is_value_truthy, Function, RV},
        util::alloc_shared,
    };

    #[test]
    fn test_is_value_truthy() {
        assert_eq!(is_value_truthy(RV::Null), false);
        assert_eq!(is_value_truthy(RV::Undefined), false);
        assert_eq!(is_value_truthy(RV::NaN), false);
        assert_eq!(is_value_truthy(RV::Bool(false)), false);
        assert_eq!(is_value_truthy(RV::Bool(true)), true);
        assert_eq!(is_value_truthy(RV::Num(0.0)), false);
        assert_eq!(is_value_truthy(RV::Num(0.1)), true);
        assert_eq!(is_value_truthy(RV::Num(1.0)), true);
        assert_eq!(is_value_truthy(RV::Num(0.0)), false);
        assert_eq!(is_value_truthy(RV::Num(-1.0)), true);
        assert_eq!(is_value_truthy(RV::Str(Arc::new("".to_owned()))), false);
        assert_eq!(is_value_truthy(RV::Str(Arc::new("0".to_owned()))), false);
        assert_eq!(is_value_truthy(RV::Str(Arc::new("false".to_owned()))), true);
        assert_eq!(is_value_truthy(RV::Str(Arc::new("true".to_owned()))), true);
        assert_eq!(is_value_truthy(RV::Str(Arc::new("foo".to_owned()))), true);
        assert_eq!(is_value_truthy(RV::Array(alloc_shared(vec![]))), true);
        assert_eq!(
            is_value_truthy(RV::Object(alloc_shared(FxHashMap::default()))),
            true
        );
        assert_eq!(
            is_value_truthy(RV::Callable(
                Some(1),
                Arc::new(Function::Lambda {
                    function: |_, _| Ok(RV::Undefined)
                })
            )),
            true
        );
    }

    #[test]
    fn test_coerce2number() {
        assert_eq!(coerce2number(RV::Num(1.0)), Some(1.0));
        assert_eq!(coerce2number(RV::Bool(false)), Some(0.0));
        assert_eq!(coerce2number(RV::Bool(true)), Some(1.0));
        assert_eq!(coerce2number(RV::Str(Arc::new("".to_owned()))), None);
    }
}