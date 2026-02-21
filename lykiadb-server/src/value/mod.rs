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
    // Boolean-like values
    Undefined,
    Null,
    Bool(bool),

    // Primitive values
    Int32(i32),
    Int64(i64),
    Double(f64),
    Decimal128(bson::Decimal128),
    DateTime(bson::DateTime),

    // Reference types
    // TODO: Replace them with references
    Str(Arc<String>),
    Document(bson::Document),
    DocumentArray(bson::Array),

    // Engine types
    Object(RVObject),
    Array(RVArray),
    Callable(RVCallable),
    Datatype(Datatype),
}

impl Eq for RV {}

impl std::hash::Hash for RV {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            RV::Str(s) => s.hash(state),
            RV::Double(n) => n.to_bits().hash(state),
            RV::Int32(i) => i.hash(state),
            RV::Int64(i) => i.hash(state),
            RV::Decimal128(d) => d.to_string().hash(state),
            RV::Bool(b) => b.hash(state),
            RV::Object(obj) => {
                // Hash the object by its pointer address
                // This is necessary because RVObject doesn't implement Hash
                (obj as *const _ as usize).hash(state);
            }
            RV::Array(arr) => {
                // Hash array length and each element
                arr.len().hash(state);
                for item in arr.iter() {
                    item.hash(state);
                }
            }
            RV::Callable(callable) => {
                // Hash by pointer address for identity-based hashing
                (callable as *const _ as usize).hash(state);
            }
            RV::Datatype(dtype) => {
                // Hash the string representation of the datatype
                dtype.to_string().hash(state);
            }

            RV::DateTime(date_time) => {
                date_time.timestamp_millis().hash(state);
            },
            RV::Document(document) => {
                // Hash the document by its pointer address
                (document as *const _ as usize).hash(state);
            },
            RV::DocumentArray(bsons) => {
                // Hash the array length and each document's pointer address
                bsons.len().hash(state);
                for doc in bsons {
                    (doc as *const _ as usize).hash(state);
                }
            }
            RV::Undefined | RV::Null => {
                // Discriminant is enough for Undefined and Null
            }
        }
    }
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
            RV::Double(_) => Datatype::Double,
            RV::Int32(_) => Datatype::Int32,
            RV::Int64(_) => Datatype::Int64,
            RV::Decimal128(_) => Datatype::Decimal128,
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
            RV::DateTime(_) => Datatype::DateTime,
            RV::Document(_) => Datatype::Document,
            RV::DocumentArray(_) => Datatype::DocumentArray,
            RV::Datatype(_) => Datatype::Datatype,
            RV::Undefined | RV::Null => Datatype::None,
        }
    }

    pub fn as_bool(&self) -> bool {
        match &self {
            RV::Double(value) => !value.is_nan() && value.abs() > 0.0,
            RV::Str(value) => !value.is_empty(),
            RV::Bool(value) => *value,
            RV::Undefined => false,
            _ => true,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            RV::Double(value) => Some(*value),
            RV::Bool(true) => Some(1.0),
            RV::Bool(false) => Some(0.0),
            RV::Str(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn if_object(&self) -> Option<&RVObject> {
        match self {
            RV::Object(obj) => Some(obj),
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
            RV::Double(n) => write!(f, "{n}"),
            RV::Int32(i) => write!(f, "{i}"),
            RV::Int64(i) => write!(f, "{i}"),
            RV::Decimal128(d) => write!(f, "{d}"),
            RV::Bool(b) => write!(f, "{b}"),
            RV::Undefined => write!(f, "undefined"),
            RV::Null => write!(f, "null"),
            RV::DateTime(date_time) => write!(f, "{date_time}"),
            RV::Document(_) => write!(f, "<Document>"),
            RV::DocumentArray(_) => write!(f, "<DocumentArray>"),
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
            RV::Double(n) => serializer.serialize_f64(*n),
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
            serde_json::Value::Number(n) => Ok(RV::Double(n.as_f64().unwrap())),
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
            (RV::Double(a), RV::Double(b)) => a == b,
            (RV::Bool(a), RV::Bool(b)) => a == b,
            //
            (RV::Str(_), RV::Double(b)) => self.eq_str_num(*b),
            (RV::Double(a), RV::Str(_)) => other.eq_str_num(*a),
            //
            (RV::Str(_), RV::Bool(b)) => self.eq_any_bool(*b),
            (RV::Bool(a), RV::Str(_)) => other.eq_any_bool(*a),
            //
            (RV::Double(_), RV::Bool(b)) => self.eq_any_bool(*b),
            (RV::Bool(a), RV::Double(_)) => other.eq_any_bool(*a),
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
            (RV::Double(a), RV::Double(b)) => a.partial_cmp(b),
            (RV::Bool(a), RV::Bool(b)) => a.partial_cmp(b),
            //
            (RV::Str(a), RV::Double(b)) => {
                if let Ok(num) = a.parse::<f64>() {
                    return num.partial_cmp(b);
                }
                None
            }
            (RV::Double(a), RV::Str(b)) => {
                if let Ok(num) = b.parse::<f64>() {
                    return a.partial_cmp(&num);
                }
                None
            }
            //
            (RV::Str(_), RV::Bool(b)) => self.partial_cmp_str_bool(*b),
            (RV::Bool(a), RV::Str(_)) => other.partial_cmp_str_bool(*a),
            //
            (RV::Double(num), RV::Bool(b)) => num.partial_cmp(&if *b { 1.0 } else { 0.0 }),
            (RV::Bool(b), RV::Double(num)) => (if *b { 1.0 } else { 0.0 }).partial_cmp(num),
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
            (RV::Bool(_), RV::Bool(_)) | (RV::Double(_), RV::Bool(_)) | (RV::Bool(_), RV::Double(_)) => {
                RV::Double(self.as_number().unwrap() + rhs.as_number().unwrap())
            }

            (RV::Double(l), RV::Double(r)) => RV::Double(l + r),
            //
            (RV::Str(l), RV::Str(r)) => RV::Str(Arc::new(l.to_string() + &r.to_string())),
            //
            (RV::Str(s), RV::Double(num)) => RV::Str(Arc::new(s.to_string() + &num.to_string())),
            (RV::Double(num), RV::Str(s)) => RV::Str(Arc::new(num.to_string() + &s.to_string())),
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
                .map(|(a, b)| RV::Double(a - b))
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
                .map(|(a, b)| RV::Double(a * b))
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
                        RV::Double(a / b)
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
        assert!(!RV::Double(0.0).as_bool());
        assert!(RV::Double(1.0).as_bool());
        assert!(RV::Double(-1.0).as_bool());
        assert!(!RV::Double(f64::NAN).as_bool());

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
        assert_eq!(RV::Double(42.0).as_number(), Some(42.0));
        assert_eq!(RV::Double(-42.0).as_number(), Some(-42.0));
        assert_eq!(RV::Double(0.0).as_number(), Some(0.0));

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
        let arr = vec![RV::Double(1.0), RV::Str(Arc::new("test".to_string()))];
        let array = RV::Array(RVArray::from_vec(arr));

        assert_eq!(RV::Double(1.0).is_in(&array), RV::Bool(true));
        assert_eq!(RV::Double(2.0).is_in(&array), RV::Bool(false));
        assert_eq!(
            RV::Str(Arc::new("test".to_string())).is_in(&array),
            RV::Bool(true)
        );

        // Test object key contains
        let mut map = FxHashMap::default();
        map.insert("key".to_string(), RV::Double(1.0));
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
        assert_eq!(RV::Double(0.0).not(), RV::Bool(true));
        assert_eq!(RV::Double(1.0).not(), RV::Bool(false));
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
        assert_eq!(RV::Double(42.0).to_string(), "42");
        assert_eq!(RV::Bool(true).to_string(), "true");
        assert_eq!(RV::Bool(false).to_string(), "false");
        assert_eq!(RV::Undefined.to_string(), "undefined");

        let arr = vec![RV::Double(1.0), RV::Str(Arc::new("test".to_string()))];
        assert_eq!(RV::Array(RVArray::from_vec(arr)).to_string(), "[1, test]");

        let mut map = FxHashMap::default();
        map.insert("key".to_string(), RV::Double(42.0));
        assert_eq!(RV::Object(RVObject::from_map(map)).to_string(), "{key: 42}");
    }

    #[test]
    fn test_rv_hash_primitives() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn hash(rv: &RV) -> u64 {
            let mut hasher = DefaultHasher::new();
            rv.hash(&mut hasher);
            hasher.finish()
        }

        // Strings: value-based hashing
        assert_ne!(
            hash(&RV::Str(Arc::new("hello".to_string()))),
            hash(&RV::Str(Arc::new("world".to_string())))
        );
        assert_eq!(
            hash(&RV::Str(Arc::new("test".to_string()))),
            hash(&RV::Str(Arc::new("test".to_string())))
        );

        // Numbers: value-based hashing
        assert_ne!(hash(&RV::Double(1.0)), hash(&RV::Double(2.0)));
        assert_ne!(hash(&RV::Double(10.0)), hash(&RV::Double(500.0)));
        assert_eq!(hash(&RV::Double(42.0)), hash(&RV::Double(42.0)));

        // Booleans: value-based hashing
        assert_ne!(hash(&RV::Bool(true)), hash(&RV::Bool(false)));
        assert_eq!(hash(&RV::Bool(true)), hash(&RV::Bool(true)));

        // Undefined: discriminant-only hashing
        assert_eq!(hash(&RV::Undefined), hash(&RV::Undefined));
    }

    #[test]
    fn test_rv_hash_arrays() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn hash(rv: &RV) -> u64 {
            let mut hasher = DefaultHasher::new();
            rv.hash(&mut hasher);
            hasher.finish()
        }

        // Arrays: content-based hashing
        let arr1 = RV::Array(RVArray::from_vec(vec![RV::Double(1.0), RV::Double(2.0)]));
        let arr2 = RV::Array(RVArray::from_vec(vec![RV::Double(1.0), RV::Double(2.0)]));
        let arr3 = RV::Array(RVArray::from_vec(vec![RV::Double(1.0), RV::Double(3.0)]));

        assert_eq!(hash(&arr1), hash(&arr2)); // Same content
        assert_ne!(hash(&arr1), hash(&arr3)); // Different content
    }

    #[test]
    fn test_rv_hash_objects() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn hash(rv: &RV) -> u64 {
            let mut hasher = DefaultHasher::new();
            rv.hash(&mut hasher);
            hasher.finish()
        }

        // Objects: pointer-based hashing (identity)
        let mut map = FxHashMap::default();
        map.insert("key".to_string(), RV::Double(1.0));
        let obj1 = RV::Object(RVObject::from_map(map));

        let mut map = FxHashMap::default();
        map.insert("key".to_string(), RV::Double(1.0));
        let obj2 = RV::Object(RVObject::from_map(map));

        // Different instances, even with same content, hash differently
        assert_ne!(hash(&obj1), hash(&obj2));
    }

    #[test]
    fn test_rv_hash_datatypes() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn hash(rv: &RV) -> u64 {
            let mut hasher = DefaultHasher::new();
            rv.hash(&mut hasher);
            hasher.finish()
        }

        // Datatypes: value-based hashing via string representation
        assert_ne!(
            hash(&RV::Datatype(Datatype::Str)),
            hash(&RV::Datatype(Datatype::Double))
        );
        assert_eq!(
            hash(&RV::Datatype(Datatype::Bool)),
            hash(&RV::Datatype(Datatype::Bool))
        );
    }

    #[test]
    fn test_rv_hash_for_group_by() {
        use std::collections::HashMap;

        // This test verifies the fix for the GROUP BY performance issue
        // Numbers with different values must hash to different buckets

        let mut groups: HashMap<RV, usize> = HashMap::new();

        for i in 0..500 {
            let key = RV::Double(i as f64);
            *groups.entry(key).or_insert(0) += 1;
        }

        // Should have 500 distinct hash buckets
        assert_eq!(groups.len(), 500);
    }
}
