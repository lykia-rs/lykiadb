use serde::{Deserialize, Serialize, Serializer};
use std::rc::Rc;
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub enum CallableError {
    GenericError { line: u32, chr: char },
}

pub trait Callable {
    fn call(&self, args: Vec<RV>) -> Result<RV, CallableError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum RV {
    Str(Rc<String>),
    Num(f64),
    Bool(bool),
    Object(FxHashMap<String, RV>),
    Array(Vec<RV>),
    Callable(Option<usize>, Rc<dyn Callable>),
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

impl<'se> Serialize for RV {
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
