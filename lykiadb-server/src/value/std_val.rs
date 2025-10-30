use lykiadb_lang::types::Datatype;
use rustc_hash::FxHashMap;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::{Arc, RwLock};
use std::ops;

use crate::util::Shared;
use crate::util::alloc_shared;
use crate::value::callable::Callable;
use crate::value::Value;

#[derive(Debug, Clone)]
pub enum StdVal {
    Str(Arc<String>),
    Num(f64),
    Bool(bool),
    Object(Shared<FxHashMap<String, StdVal>>),
    Array(Shared<Vec<StdVal>>),
    Callable(Callable<StdVal>),
    Datatype(Datatype),
    Undefined,
}

impl Value for StdVal {
    type Array = Shared<Vec<StdVal>>;
    type Object = Shared<FxHashMap<String, StdVal>>;

    fn datatype(dt: Datatype) -> Self {
        StdVal::Datatype(dt)
    }

    fn string(s: String) -> Self {
        StdVal::Str(Arc::new(s))
    }

    fn number(n: f64) -> Self {
        StdVal::Num(n)
    }

    fn boolean(b: bool) -> Self {
        StdVal::Bool(b)
    }

    fn array(arr: Vec<StdVal>) -> Self {
        StdVal::Array(alloc_shared(arr))
    }

    fn object(obj: FxHashMap<String, StdVal>) -> Self {
        StdVal::Object(alloc_shared(obj))
    }

    fn callable(c: Callable<Self>) -> Self {
        StdVal::Callable(c)
    }

    fn undefined() -> Self {
        StdVal::Undefined
    }
    
    fn get_type(&self) -> Datatype {
        self.get_type()
    }
    
    fn as_bool(&self) -> bool {
        self.as_bool()
    }
    
    fn as_number(&self) -> Option<f64> {
        self.as_number()
    }
    
    fn as_string(&self) -> Option<String> {
        self.as_string()
    }

    fn as_callable(&self) -> Option<&Callable<Self>> {
        match self {
            StdVal::Callable(c) => Some(c),
            _ => None,
        }
    }

    fn as_datatype(&self) -> Option<&Datatype> {
        match self {
            StdVal::Datatype(dt) => Some(dt),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<&<StdVal as Value>::Object> {
        match self {
            StdVal::Object(obj) => Some(obj),
            _ => None,
        }
    }
    
    fn is_in(&self, other: &Self) -> Self {
        self.is_in(other)
    }
    
    fn eq_str_num(&self, n: f64) -> bool {
        self.eq_str_num(n)
    }
    
    fn partial_cmp_str_bool(&self, other: bool) -> Option<std::cmp::Ordering> {
        self.partial_cmp_str_bool(other)
    }
}

impl From<StdVal> for Datatype {
    fn from(rv: StdVal) -> Self {
        match rv {
            StdVal::Datatype(t) => t,
            _ => Datatype::None,
        }
    }
}

impl StdVal {
    pub fn get_type(&self) -> Datatype {
        match &self {
            StdVal::Str(_) => Datatype::Str,
            StdVal::Num(_) => Datatype::Num,
            StdVal::Bool(_) => Datatype::Bool,
            StdVal::Object(obj) => {
                let obj: &FxHashMap<String, StdVal> = &obj.read().unwrap();
                if obj.is_empty() {
                    return Datatype::None;
                }
                let mut object = FxHashMap::default();
                for key in obj.keys() {
                    let datatype = obj.get(key).unwrap().get_type();
                    object.insert(key.to_string(), datatype);
                }
                Datatype::Object(object)
            }
            StdVal::Array(arr) => {
                let arr: &[StdVal] = &arr.read().unwrap();
                if arr.is_empty() {
                    return Datatype::Array(Box::from(Datatype::None));
                }
                Datatype::Array(Box::from(arr[0].get_type()))
            }
            StdVal::Callable(c) => {
                let input = Box::from(c.parameter_types.clone());
                let output = Box::from(c.return_type.clone());
                Datatype::Callable(input, output)
            }
            StdVal::Datatype(_) => Datatype::Datatype,
            StdVal::Undefined => Datatype::None,
        }
    }

    pub fn as_bool(&self) -> bool {
        match &self {
            StdVal::Num(value) => !value.is_nan() && value.abs() > 0.0,
            StdVal::Str(value) => !value.is_empty(),
            StdVal::Bool(value) => *value,
            StdVal::Undefined => false,
            _ => true,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            StdVal::Num(value) => Some(*value),
            StdVal::Bool(true) => Some(1.0),
            StdVal::Bool(false) => Some(0.0),
            StdVal::Str(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            StdVal::Str(s) => Some((**s).clone()),
            StdVal::Num(n) => Some(n.to_string()),
            StdVal::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }

    pub fn eq_any_bool(&self, b: bool) -> bool {
        self.as_bool() == b
    }

    pub fn eq_str_num(&self, n: f64) -> bool {
        if let StdVal::Str(s) = self
            && let Ok(num) = s.parse::<f64>()
        {
            return num == n;
        }
        false
    }

    pub fn partial_cmp_str_bool(&self, other: bool) -> Option<std::cmp::Ordering> {
        if let Some(num) = self.as_number() {
            return num.partial_cmp(&if other { 1.0 } else { 0.0 });
        }
        self.as_bool().partial_cmp(&other)
    }

    pub fn is_in(&self, other: &StdVal) -> StdVal {
        match (self, other) {
            (StdVal::Str(lhs), StdVal::Str(rhs)) => StdVal::Bool(rhs.contains(lhs.as_str())),
            (lhs, StdVal::Array(rhs)) => StdVal::Bool(rhs.read().unwrap().contains(lhs)),
            (StdVal::Str(key), StdVal::Object(map)) => {
                StdVal::Bool(map.read().unwrap().contains_key(key.as_str()))
            }
            _ => StdVal::Bool(false),
        }
    }

    pub fn not(&self) -> StdVal {
        StdVal::Bool(!self.as_bool())
    }
}

impl PartialEq for StdVal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StdVal::Array(_), StdVal::Array(_)) | (StdVal::Object(_), StdVal::Object(_)) => false,
            (StdVal::Undefined, StdVal::Undefined) => true,
            //
            (StdVal::Undefined, _) | (_, StdVal::Undefined) => false,
            //
            (StdVal::Str(a), StdVal::Str(b)) => a == b,
            (StdVal::Num(a), StdVal::Num(b)) => a == b,
            (StdVal::Bool(a), StdVal::Bool(b)) => a == b,
            //
            (StdVal::Str(_), StdVal::Num(b)) => self.eq_str_num(*b),
            (StdVal::Num(a), StdVal::Str(_)) => other.eq_str_num(*a),
            //
            (StdVal::Str(_), StdVal::Bool(b)) => self.eq_any_bool(*b),
            (StdVal::Bool(a), StdVal::Str(_)) => other.eq_any_bool(*a),
            //
            (StdVal::Num(_), StdVal::Bool(b)) => self.eq_any_bool(*b),
            (StdVal::Bool(a), StdVal::Num(_)) => other.eq_any_bool(*a),
            //
            (StdVal::Datatype(a), StdVal::Datatype(b)) => a == b,
            //
            _ => false,
        }
    }
}

impl PartialOrd for StdVal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (StdVal::Array(_), StdVal::Array(_)) | (StdVal::Object(_), StdVal::Object(_)) => None,
            (StdVal::Undefined, StdVal::Undefined) => Some(std::cmp::Ordering::Equal),
            //
            (StdVal::Undefined, _) | (_, StdVal::Undefined) => None,
            //
            (StdVal::Str(a), StdVal::Str(b)) => Some(a.cmp(b)),
            (StdVal::Num(a), StdVal::Num(b)) => a.partial_cmp(b),
            (StdVal::Bool(a), StdVal::Bool(b)) => a.partial_cmp(b),
            //
            (StdVal::Str(a), StdVal::Num(b)) => {
                if let Ok(num) = a.parse::<f64>() {
                    return num.partial_cmp(b);
                }
                None
            }
            (StdVal::Num(a), StdVal::Str(b)) => {
                if let Ok(num) = b.parse::<f64>() {
                    return a.partial_cmp(&num);
                }
                None
            }
            //
            (StdVal::Str(_), StdVal::Bool(b)) => self.partial_cmp_str_bool(*b),
            (StdVal::Bool(a), StdVal::Str(_)) => other.partial_cmp_str_bool(*a),
            //
            (StdVal::Num(num), StdVal::Bool(b)) => num.partial_cmp(&if *b { 1.0 } else { 0.0 }),
            (StdVal::Bool(b), StdVal::Num(num)) => (if *b { 1.0 } else { 0.0 }).partial_cmp(num),
            //
            _ => None,
        }
    }
}

impl ops::Add for StdVal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            //
            (StdVal::Bool(_), StdVal::Bool(_)) | (StdVal::Num(_), StdVal::Bool(_)) | (StdVal::Bool(_), StdVal::Num(_)) => {
                StdVal::Num(self.as_number().unwrap() + rhs.as_number().unwrap())
            }

            (StdVal::Num(l), StdVal::Num(r)) => StdVal::Num(l + r),
            //
            (StdVal::Str(l), StdVal::Str(r)) => StdVal::Str(Arc::new(l.to_string() + &r.to_string())),
            //
            (StdVal::Str(s), StdVal::Num(num)) => StdVal::Str(Arc::new(s.to_string() + &num.to_string())),
            (StdVal::Num(num), StdVal::Str(s)) => StdVal::Str(Arc::new(num.to_string() + &s.to_string())),
            //
            (StdVal::Str(s), StdVal::Bool(bool)) => StdVal::Str(Arc::new(s.to_string() + &bool.to_string())),
            (StdVal::Bool(bool), StdVal::Str(s)) => StdVal::Str(Arc::new(bool.to_string() + &s.to_string())),
            //
            (_, _) => StdVal::Undefined,
        }
    }
}

impl ops::Sub for StdVal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (StdVal::Undefined, _) | (_, StdVal::Undefined) => StdVal::Undefined,
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| StdVal::Num(a - b))
                .unwrap_or(StdVal::Undefined),
        }
    }
}

impl ops::Mul for StdVal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (StdVal::Undefined, _) | (_, StdVal::Undefined) => StdVal::Undefined,
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| StdVal::Num(a * b))
                .unwrap_or(StdVal::Undefined),
        }
    }
}

impl ops::Div for StdVal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (StdVal::Undefined, _) | (_, StdVal::Undefined) => StdVal::Undefined,
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| {
                    if a == 0.0 && b == 0.0 {
                        StdVal::Undefined
                    } else {
                        StdVal::Num(a / b)
                    }
                })
                .unwrap_or(StdVal::Undefined),
        }
    }
}

impl Display for StdVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StdVal::Str(s) => write!(f, "{s}"),
            StdVal::Num(n) => write!(f, "{n}"),
            StdVal::Bool(b) => write!(f, "{b}"),
            StdVal::Undefined => write!(f, "undefined"),
            StdVal::Array(arr) => {
                let arr = (arr as &RwLock<Vec<StdVal>>).read().unwrap();
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            StdVal::Object(obj) => {
                let obj = (obj as &RwLock<FxHashMap<String, StdVal>>).read().unwrap();
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{key}: {value}")?;
                }
                write!(f, "}}")
            }
            StdVal::Callable(_) => write!(f, "<Callable>"),
            StdVal::Datatype(dtype) => write!(f, "<Datatype, {dtype}>"),
        }
    }
}

impl Serialize for StdVal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            StdVal::Str(s) => serializer.serialize_str(s),
            StdVal::Num(n) => serializer.serialize_f64(*n),
            StdVal::Bool(b) => serializer.serialize_bool(*b),
            StdVal::Undefined => serializer.serialize_none(),
            StdVal::Array(arr) => {
                let mut seq = serializer.serialize_seq(None).unwrap();
                let arr = (arr as &RwLock<Vec<StdVal>>).read().unwrap();
                for item in arr.iter() {
                    seq.serialize_element(&item)?;
                }
                seq.end()
            }
            StdVal::Object(obj) => {
                let mut map = serializer.serialize_map(None).unwrap();
                let arr = (obj as &RwLock<FxHashMap<String, StdVal>>).read().unwrap();
                for (key, value) in arr.iter() {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
            _ => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for StdVal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(s) => Ok(StdVal::Str(Arc::new(s))),
            serde_json::Value::Number(n) => Ok(StdVal::Num(n.as_f64().unwrap())),
            serde_json::Value::Bool(b) => Ok(StdVal::Bool(b)),
            serde_json::Value::Array(arr) => {
                let mut vec = vec![];
                for item in arr {
                    vec.push(serde_json::from_value(item).unwrap());
                }
                Ok(StdVal::Array(alloc_shared(vec)))
            }
            serde_json::Value::Object(obj) => {
                let mut map = FxHashMap::default();
                for (key, value) in obj {
                    map.insert(key, serde_json::from_value(value).unwrap());
                }
                Ok(StdVal::Object(alloc_shared(map)))
            }
            serde_json::Value::Null => Ok(StdVal::Undefined),
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
        assert!(!StdVal::Num(0.0).as_bool());
        assert!(StdVal::Num(1.0).as_bool());
        assert!(StdVal::Num(-1.0).as_bool());
        assert!(!StdVal::Num(f64::NAN).as_bool());

        // Test strings
        assert!(!StdVal::Str(Arc::new("".to_string())).as_bool());
        assert!(StdVal::Str(Arc::new("hello".to_string())).as_bool());

        // Test booleans
        assert!(StdVal::Bool(true).as_bool());
        assert!(!StdVal::Bool(false).as_bool());

        // Test special values
        assert!(!StdVal::Undefined.as_bool());

        // Test collections
        let empty_array = StdVal::Array(alloc_shared(Vec::new()));
        let empty_object = StdVal::Object(alloc_shared(FxHashMap::default()));
        assert!(empty_array.as_bool());
        assert!(empty_object.as_bool());
    }

    #[test]
    fn test_rv_as_number() {
        // Test numeric values
        assert_eq!(StdVal::Num(42.0).as_number(), Some(42.0));
        assert_eq!(StdVal::Num(-42.0).as_number(), Some(-42.0));
        assert_eq!(StdVal::Num(0.0).as_number(), Some(0.0));

        // Test booleans
        assert_eq!(StdVal::Bool(true).as_number(), Some(1.0));
        assert_eq!(StdVal::Bool(false).as_number(), Some(0.0));

        // Test strings
        assert_eq!(StdVal::Str(Arc::new("42".to_string())).as_number(), Some(42.0));
        assert_eq!(
            StdVal::Str(Arc::new("-42".to_string())).as_number(),
            Some(-42.0)
        );
        assert_eq!(StdVal::Str(Arc::new("invalid".to_string())).as_number(), None);
        assert_eq!(StdVal::Str(Arc::new("".to_string())).as_number(), None);

        // Test other types
        assert_eq!(StdVal::Undefined.as_number(), None);
        assert_eq!(StdVal::Array(alloc_shared(Vec::new())).as_number(), None);
        assert_eq!(
            StdVal::Object(alloc_shared(FxHashMap::default())).as_number(),
            None
        );
    }

    #[test]
    fn test_rv_is_in() {
        // Test string contains
        let haystack = StdVal::Str(Arc::new("hello world".to_string()));
        let needle = StdVal::Str(Arc::new("world".to_string()));
        assert_eq!(needle.is_in(&haystack), StdVal::Bool(true));

        let not_found = StdVal::Str(Arc::new("xyz".to_string()));
        assert_eq!(not_found.is_in(&haystack), StdVal::Bool(false));

        // Test array contains
        let arr = vec![StdVal::Num(1.0), StdVal::Str(Arc::new("test".to_string()))];
        let array = StdVal::Array(alloc_shared(arr));

        assert_eq!(StdVal::Num(1.0).is_in(&array), StdVal::Bool(true));
        assert_eq!(StdVal::Num(2.0).is_in(&array), StdVal::Bool(false));
        assert_eq!(
            StdVal::Str(Arc::new("test".to_string())).is_in(&array),
            StdVal::Bool(true)
        );

        // Test object key contains
        let mut map = FxHashMap::default();
        map.insert("key".to_string(), StdVal::Num(1.0));
        let object = StdVal::Object(alloc_shared(map));

        assert_eq!(
            StdVal::Str(Arc::new("key".to_string())).is_in(&object),
            StdVal::Bool(true)
        );
        assert_eq!(
            StdVal::Str(Arc::new("missing".to_string())).is_in(&object),
            StdVal::Bool(false)
        );
    }

    #[test]
    fn test_rv_not() {
        assert_eq!(StdVal::Bool(true).not(), StdVal::Bool(false));
        assert_eq!(StdVal::Bool(false).not(), StdVal::Bool(true));
        assert_eq!(StdVal::Num(0.0).not(), StdVal::Bool(true));
        assert_eq!(StdVal::Num(1.0).not(), StdVal::Bool(false));
        assert_eq!(StdVal::Str(Arc::new("".to_string())).not(), StdVal::Bool(true));
        assert_eq!(
            StdVal::Str(Arc::new("hello".to_string())).not(),
            StdVal::Bool(false)
        );
        assert_eq!(StdVal::Undefined.not(), StdVal::Bool(true));
    }

    #[test]
    fn test_rv_display() {
        assert_eq!(StdVal::Str(Arc::new("hello".to_string())).to_string(), "hello");
        assert_eq!(StdVal::Num(42.0).to_string(), "42");
        assert_eq!(StdVal::Bool(true).to_string(), "true");
        assert_eq!(StdVal::Bool(false).to_string(), "false");
        assert_eq!(StdVal::Undefined.to_string(), "undefined");

        let arr = vec![StdVal::Num(1.0), StdVal::Str(Arc::new("test".to_string()))];
        assert_eq!(StdVal::Array(alloc_shared(arr)).to_string(), "[1, test]");

        let mut map = FxHashMap::default();
        map.insert("key".to_string(), StdVal::Num(42.0));
        assert_eq!(StdVal::Object(alloc_shared(map)).to_string(), "{key: 42}");
    }
}
