use datatype::Datatype;
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
pub mod datatype;

#[derive(Debug, Clone)]
pub enum RV {
    Str(Arc<String>),
    Num(f64),
    Bool(bool),
    Object(Shared<FxHashMap<String, RV>>),
    Array(Shared<Vec<RV>>),
    Callable(Callable),
    Datatype(Datatype),
    Undefined
}

impl RV {
    pub fn get_type(&self) -> Datatype {
        match &self {
            RV::Str(_) => Datatype::Str,
            RV::Num(_) => Datatype::Num,
            RV::Bool(_) => Datatype::Bool,
            RV::Object(obj) => {
                let obj: &FxHashMap<String, RV> = &obj.read().unwrap();
                if obj.is_empty() {
                    return Datatype::None;
                }
                let mut document = FxHashMap::default();
                for key in obj.keys() {
                    let datatype = obj.get(key).unwrap().get_type();
                    document.insert(key.to_string(), datatype);
                }
                Datatype::Document(document)
            },
            RV::Array(arr) => {
                let arr: &[RV] = &arr.read().unwrap();
                if arr.is_empty() {
                    return Datatype::Array(Box::from(Datatype::None));
                }
                Datatype::Array(Box::from(arr[0].get_type()))
            },
            RV::Callable(_) => Datatype::Callable,
            RV::Datatype(_) => Datatype::Datatype,
            RV::Undefined => Datatype::None,
        }
    }

    pub fn as_bool(&self) -> bool {
        match &self {
            RV::Num(value) => !value.is_nan() && value.abs() > 0.0,
            RV::Str(value) => !value.is_empty(),
            RV::Bool(value) => *value,
            RV::Undefined => false,
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
            RV::Datatype(dtype) => write!(f, "<Datatype, {}>", dtype),
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
                let mut vec = vec![];
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
            serde_json::Value::Null => Ok(RV::Undefined),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_rv_as_bool() {
        // Test numeric values
        assert!(!RV::Num(0.0).as_bool());
        assert!(RV::Num(1.0).as_bool());
        assert!(RV::Num(-1.0).as_bool());
        assert!(!RV::Num(f64::NAN).as_bool());

        // Test strings
        assert!(!RV::Str(Arc::new("".to_string())).as_bool());
        assert!(RV::Str(Arc::new("hello".to_string())).as_bool());

        // Test booleans
        assert!(RV::Bool(true).as_bool());
        assert!(!RV::Bool(false).as_bool());

        // Test special values
        assert!(!RV::Undefined.as_bool());

        // Test collections
        let empty_array = RV::Array(alloc_shared(Vec::new()));
        let empty_object = RV::Object(alloc_shared(FxHashMap::default()));
        assert!(empty_array.as_bool());
        assert!(empty_object.as_bool());
    }

    #[test]
    fn test_rv_as_number() {
        // Test numeric values
        assert_eq!(RV::Num(42.0).as_number(), Some(42.0));
        assert_eq!(RV::Num(-42.0).as_number(), Some(-42.0));
        assert_eq!(RV::Num(0.0).as_number(), Some(0.0));

        // Test booleans
        assert_eq!(RV::Bool(true).as_number(), Some(1.0));
        assert_eq!(RV::Bool(false).as_number(), Some(0.0));

        // Test strings
        assert_eq!(RV::Str(Arc::new("42".to_string())).as_number(), Some(42.0));
        assert_eq!(
            RV::Str(Arc::new("-42".to_string())).as_number(),
            Some(-42.0)
        );
        assert_eq!(RV::Str(Arc::new("invalid".to_string())).as_number(), None);
        assert_eq!(RV::Str(Arc::new("".to_string())).as_number(), None);

        // Test other types
        assert_eq!(RV::Undefined.as_number(), None);
        assert_eq!(RV::Array(alloc_shared(Vec::new())).as_number(), None);
        assert_eq!(
            RV::Object(alloc_shared(FxHashMap::default())).as_number(),
            None
        );
    }

    #[test]
    fn test_rv_is_in() {
        // Test string contains
        let haystack = RV::Str(Arc::new("hello world".to_string()));
        let needle = RV::Str(Arc::new("world".to_string()));
        assert_eq!(needle.is_in(&haystack), RV::Bool(true));

        let not_found = RV::Str(Arc::new("xyz".to_string()));
        assert_eq!(not_found.is_in(&haystack), RV::Bool(false));

        // Test array contains
        let arr = vec![RV::Num(1.0), RV::Str(Arc::new("test".to_string()))];
        let array = RV::Array(alloc_shared(arr));

        assert_eq!(RV::Num(1.0).is_in(&array), RV::Bool(true));
        assert_eq!(RV::Num(2.0).is_in(&array), RV::Bool(false));
        assert_eq!(
            RV::Str(Arc::new("test".to_string())).is_in(&array),
            RV::Bool(true)
        );

        // Test object key contains
        let mut map = FxHashMap::default();
        map.insert("key".to_string(), RV::Num(1.0));
        let object = RV::Object(alloc_shared(map));

        assert_eq!(
            RV::Str(Arc::new("key".to_string())).is_in(&object),
            RV::Bool(true)
        );
        assert_eq!(
            RV::Str(Arc::new("missing".to_string())).is_in(&object),
            RV::Bool(false)
        );
    }

    #[test]
    fn test_rv_not() {
        assert_eq!(RV::Bool(true).not(), RV::Bool(false));
        assert_eq!(RV::Bool(false).not(), RV::Bool(true));
        assert_eq!(RV::Num(0.0).not(), RV::Bool(true));
        assert_eq!(RV::Num(1.0).not(), RV::Bool(false));
        assert_eq!(RV::Str(Arc::new("".to_string())).not(), RV::Bool(true));
        assert_eq!(
            RV::Str(Arc::new("hello".to_string())).not(),
            RV::Bool(false)
        );
        assert_eq!(RV::Undefined.not(), RV::Bool(true));
    }

    #[test]
    fn test_rv_display() {
        assert_eq!(RV::Str(Arc::new("hello".to_string())).to_string(), "hello");
        assert_eq!(RV::Num(42.0).to_string(), "42");
        assert_eq!(RV::Bool(true).to_string(), "true");
        assert_eq!(RV::Bool(false).to_string(), "false");
        assert_eq!(RV::Undefined.to_string(), "undefined");

        let arr = vec![RV::Num(1.0), RV::Str(Arc::new("test".to_string()))];
        assert_eq!(RV::Array(alloc_shared(arr)).to_string(), "[1, test]");

        let mut map = FxHashMap::default();
        map.insert("key".to_string(), RV::Num(42.0));
        assert_eq!(RV::Object(alloc_shared(map)).to_string(), "{key: 42}");
    }
}
