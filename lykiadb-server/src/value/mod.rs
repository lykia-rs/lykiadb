use array::RVArray;
use callable::RVCallable;
use lykiadb_lang::types::Datatype;
use object::RVObject;
use rustc_hash::FxHashMap;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops;
use std::sync::Arc;

pub mod array;
pub mod callable;
pub mod environment;
pub mod eval;
pub mod iterator;
pub mod object;

#[derive(Debug, Clone)]
pub enum RV {
    Str(Arc<String>),
    Num(f64),
    Bool(bool),
    Object(RVObject),
    Array(RVArray),
    Callable(RVCallable),
    Datatype(Datatype),
    Undefined,
}

impl From<RV> for Datatype {
    fn from(rv: RV) -> Self {
        match rv {
            RV::Datatype(t) => t,
            _ => Datatype::None,
        }
    }
}

impl RV {
    pub fn get_type(&self) -> Datatype {
        match &self {
            RV::Str(_) => Datatype::Str,
            RV::Num(_) => Datatype::Num,
            RV::Bool(_) => Datatype::Bool,
            RV::Object(obj) => {
                if obj.is_empty() {
                    return Datatype::None;
                }
                let mut object = FxHashMap::default();
                for key in obj.keys() {
                    let datatype = obj.get(&key).unwrap().get_type();
                    object.insert(key.to_string(), datatype);
                }
                Datatype::Object(object)
            }
            RV::Array(arr) => {
                if arr.is_empty() {
                    return Datatype::Array(Box::from(Datatype::None));
                }
                Datatype::Array(Box::from(arr.get(0).get_type()))
            }
            RV::Callable(c) => {
                let input = Box::from(c.parameter_types.clone());
                let output = Box::from(c.return_type.clone());
                Datatype::Callable(input, output)
            }
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
            RV::Str(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn if_object(&self) -> Option<&RVObject> {
        match self {
            RV::Object(obj) => Some(&obj),
            _ => None,
        }
    }

    pub fn eq_any_bool(&self, b: bool) -> bool {
        self.as_bool() == b
    }

    pub fn eq_str_num(&self, n: f64) -> bool {
        if let RV::Str(s) = self
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

    pub fn is_in(&self, other: &RV) -> RV {
        match (self, other) {
            (RV::Str(lhs), RV::Str(rhs)) => RV::Bool(rhs.contains(lhs.as_str())),
            (lhs, RV::Array(rhs)) => RV::Bool(rhs.contains(lhs)),
            (RV::Str(key), RV::Object(map)) => RV::Bool(map.contains_key(key.as_str())),
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
            RV::Str(s) => write!(f, "{s}"),
            RV::Num(n) => write!(f, "{n}"),
            RV::Bool(b) => write!(f, "{b}"),
            RV::Undefined => write!(f, "undefined"),
            RV::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            RV::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{key}: {value}")?;
                }
                write!(f, "}}")
            }
            RV::Callable(_) => write!(f, "<Callable>"),
            RV::Datatype(dtype) => write!(f, "<Datatype, {dtype}>"),
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
                for item in arr.iter() {
                    seq.serialize_element(&item)?;
                }
                seq.end()
            }
            RV::Object(obj) => {
                let mut map = serializer.serialize_map(None).unwrap();
                for (key, value) in obj.iter() {
                    map.serialize_entry(&key, &value)?;
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
                Ok(RV::Array(RVArray::from_vec(vec)))
            }
            serde_json::Value::Object(obj) => {
                let mut map = FxHashMap::default();
                for (key, value) in obj {
                    map.insert(key, serde_json::from_value(value).unwrap());
                }
                Ok(RV::Object(RVObject::from_map(map)))
            }
            serde_json::Value::Null => Ok(RV::Undefined),
        }
    }
}

impl PartialEq for RV {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RV::Array(_), RV::Array(_)) | (RV::Object(_), RV::Object(_)) => false,
            (RV::Undefined, RV::Undefined) => true,
            //
            (RV::Undefined, _) | (_, RV::Undefined) => false,
            //
            (RV::Str(a), RV::Str(b)) => a == b,
            (RV::Num(a), RV::Num(b)) => a == b,
            (RV::Bool(a), RV::Bool(b)) => a == b,
            //
            (RV::Str(_), RV::Num(b)) => self.eq_str_num(*b),
            (RV::Num(a), RV::Str(_)) => other.eq_str_num(*a),
            //
            (RV::Str(_), RV::Bool(b)) => self.eq_any_bool(*b),
            (RV::Bool(a), RV::Str(_)) => other.eq_any_bool(*a),
            //
            (RV::Num(_), RV::Bool(b)) => self.eq_any_bool(*b),
            (RV::Bool(a), RV::Num(_)) => other.eq_any_bool(*a),
            //
            (RV::Datatype(a), RV::Datatype(b)) => a == b,
            //
            _ => false,
        }
    }
}

impl PartialOrd for RV {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (RV::Array(_), RV::Array(_)) | (RV::Object(_), RV::Object(_)) => None,
            (RV::Undefined, RV::Undefined) => Some(std::cmp::Ordering::Equal),
            //
            (RV::Undefined, _) | (_, RV::Undefined) => None,
            //
            (RV::Str(a), RV::Str(b)) => Some(a.cmp(b)),
            (RV::Num(a), RV::Num(b)) => a.partial_cmp(b),
            (RV::Bool(a), RV::Bool(b)) => a.partial_cmp(b),
            //
            (RV::Str(a), RV::Num(b)) => {
                if let Ok(num) = a.parse::<f64>() {
                    return num.partial_cmp(b);
                }
                None
            }
            (RV::Num(a), RV::Str(b)) => {
                if let Ok(num) = b.parse::<f64>() {
                    return a.partial_cmp(&num);
                }
                None
            }
            //
            (RV::Str(_), RV::Bool(b)) => self.partial_cmp_str_bool(*b),
            (RV::Bool(a), RV::Str(_)) => other.partial_cmp_str_bool(*a),
            //
            (RV::Num(num), RV::Bool(b)) => num.partial_cmp(&if *b { 1.0 } else { 0.0 }),
            (RV::Bool(b), RV::Num(num)) => (if *b { 1.0 } else { 0.0 }).partial_cmp(num),
            //
            _ => None,
        }
    }
}

impl ops::Add for RV {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            //
            (RV::Bool(_), RV::Bool(_)) | (RV::Num(_), RV::Bool(_)) | (RV::Bool(_), RV::Num(_)) => {
                RV::Num(self.as_number().unwrap() + rhs.as_number().unwrap())
            }

            (RV::Num(l), RV::Num(r)) => RV::Num(l + r),
            //
            (RV::Str(l), RV::Str(r)) => RV::Str(Arc::new(l.to_string() + &r.to_string())),
            //
            (RV::Str(s), RV::Num(num)) => RV::Str(Arc::new(s.to_string() + &num.to_string())),
            (RV::Num(num), RV::Str(s)) => RV::Str(Arc::new(num.to_string() + &s.to_string())),
            //
            (RV::Str(s), RV::Bool(bool)) => RV::Str(Arc::new(s.to_string() + &bool.to_string())),
            (RV::Bool(bool), RV::Str(s)) => RV::Str(Arc::new(bool.to_string() + &s.to_string())),
            //
            (_, _) => RV::Undefined,
        }
    }
}

impl ops::Sub for RV {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RV::Undefined, _) | (_, RV::Undefined) => RV::Undefined,
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| RV::Num(a - b))
                .unwrap_or(RV::Undefined),
        }
    }
}

impl ops::Mul for RV {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RV::Undefined, _) | (_, RV::Undefined) => RV::Undefined,
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| RV::Num(a * b))
                .unwrap_or(RV::Undefined),
        }
    }
}

impl ops::Div for RV {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RV::Undefined, _) | (_, RV::Undefined) => RV::Undefined,
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| {
                    if a == 0.0 && b == 0.0 {
                        RV::Undefined
                    } else {
                        RV::Num(a / b)
                    }
                })
                .unwrap_or(RV::Undefined),
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
        let empty_array = RV::Array(RVArray::new());
        let empty_object = RV::Object(RVObject::new());
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
        assert_eq!(RV::Array(RVArray::new()).as_number(), None);
        assert_eq!(RV::Object(RVObject::new()).as_number(), None);
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
        let array = RV::Array(RVArray::from_vec(arr));

        assert_eq!(RV::Num(1.0).is_in(&array), RV::Bool(true));
        assert_eq!(RV::Num(2.0).is_in(&array), RV::Bool(false));
        assert_eq!(
            RV::Str(Arc::new("test".to_string())).is_in(&array),
            RV::Bool(true)
        );

        // Test object key contains
        let mut map = FxHashMap::default();
        map.insert("key".to_string(), RV::Num(1.0));
        let object = RV::Object(RVObject::from_map(map));

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
        assert_eq!(RV::Array(RVArray::from_vec(arr)).to_string(), "[1, test]");

        let mut map = FxHashMap::default();
        map.insert("key".to_string(), RV::Num(42.0));
        assert_eq!(RV::Object(RVObject::from_map(map)).to_string(), "{key: 42}");
    }
}
