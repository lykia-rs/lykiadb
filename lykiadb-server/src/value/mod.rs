use rustc_hash::FxHashMap;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::{Arc, RwLock};

use crate::util::alloc_shared;
use crate::util::Shared;
use callable::Callable;

pub mod callable;
pub mod environment;
pub mod eval;

#[derive(Debug, Clone)]
pub enum RV {
    Str(Arc<String>),
    Num(f64),
    Bool(bool),
    Object(Shared<FxHashMap<String, RV>>),
    Array(Shared<Vec<RV>>),
    Callable(Callable),
    Undefined,
    NaN,
    Null,
}

impl RV {
    pub fn as_bool(&self) -> bool {
        match &self {
            RV::Num(value) => !value.is_nan() && value.abs() > 0.0,
            RV::Str(value) => !value.is_empty(),
            RV::Bool(value) => *value,
            RV::Null | RV::Undefined | RV::NaN => false,
            _ => true,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            RV::Num(value) => Some(*value),
            RV::Bool(true) => Some(1.0),
            RV::Bool(false) => Some(0.0),
            RV::Str(s) => {
                if let Ok(num) = s.parse::<f64>() {
                    Some(num)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn eq_any_bool(&self, b: bool) -> bool {
        self.as_bool() == b
    }

    pub fn eq_str_num(&self, n: f64) -> bool {
        if let RV::Str(s) = self {
            if let Ok(num) = s.parse::<f64>() {
                return num == n;
            }
        }
        false
    }

    pub fn partial_cmp_str_bool(&self, other: bool) -> Option<std::cmp::Ordering> {
        if let Some(num) = self.as_number() {
            return num.partial_cmp(&if other { 1.0 } else { 0.0 });
        }
        self.as_bool().partial_cmp(&other)
    }

    pub fn is_in(&self, other: &RV) -> RV {
        match (self, other) {
            (RV::Str(lhs), RV::Str(rhs)) => RV::Bool(rhs.contains(lhs.as_str())),
            (lhs, RV::Array(rhs)) => RV::Bool(rhs.read().unwrap().contains(lhs)),
            (RV::Str(key), RV::Object(map)) => {
                RV::Bool(map.read().unwrap().contains_key(key.as_str()))
            }
            _ => RV::Bool(false),
        }
    }

    pub fn not(&self) -> RV {
        RV::Bool(!self.as_bool())
    }
}

impl Display for RV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RV::Str(s) => write!(f, "{}", s),
            RV::Num(n) => write!(f, "{}", n),
            RV::Bool(b) => write!(f, "{}", b),
            RV::Undefined => write!(f, "undefined"),
            RV::NaN => write!(f, "NaN"),
            RV::Null => write!(f, "null"),
            RV::Array(arr) => {
                let arr = (arr as &RwLock<Vec<RV>>).read().unwrap();
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            RV::Object(obj) => {
                let obj = (obj as &RwLock<FxHashMap<String, RV>>).read().unwrap();
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
            RV::Callable(_) => write!(f, "<Callable>"),
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
            RV::Array(arr) => {
                let mut seq = serializer.serialize_seq(None).unwrap();
                let arr = (arr as &RwLock<Vec<RV>>).read().unwrap();
                for item in arr.iter() {
                    seq.serialize_element(&item)?;
                }
                seq.end()
            }
            RV::Object(obj) => {
                let mut map = serializer.serialize_map(None).unwrap();
                let arr = (obj as &RwLock<FxHashMap<String, RV>>).read().unwrap();
                for (key, value) in arr.iter() {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
            _ => serializer.serialize_none(),
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
            serde_json::Value::Array(arr) => {
                let mut vec = Vec::new();
                for item in arr {
                    vec.push(serde_json::from_value(item).unwrap());
                }
                Ok(RV::Array(alloc_shared(vec)))
            }
            serde_json::Value::Object(obj) => {
                let mut map = FxHashMap::default();
                for (key, value) in obj {
                    map.insert(key, serde_json::from_value(value).unwrap());
                }
                Ok(RV::Object(alloc_shared(map)))
            }
            serde_json::Value::Null => Ok(RV::Null),
        }
    }
}
