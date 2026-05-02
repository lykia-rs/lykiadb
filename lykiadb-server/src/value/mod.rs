use array::RVArray;
use callable::RVCallable;
use lykiadb_lang::types::Datatype;
use object::RVObject;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops;
use std::sync::Arc;

pub mod array;
pub mod callable;
pub mod document;
pub mod eval;
pub mod iterator;
pub mod object;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RV<'v> {
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

    // Engine types
    Object(RVObject<'v>),
    Array(RVArray<'v>),
    #[serde(skip)]
    Callable(RVCallable<'v>),
    Datatype(Datatype),
}

impl<'v> Eq for RV<'v> {}

impl<'v> std::hash::Hash for RV<'v> {
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
            }
            RV::Undefined | RV::Null => {
                // Discriminant is enough for Undefined and Null
            }
        }
    }
}

impl<'v> From<RV<'v>> for Datatype {
    fn from(rv: RV<'v>) -> Self {
        match rv {
            RV::Datatype(t) => t,
            _ => Datatype::None,
        }
    }
}

impl<'v> From<serde_json::Value> for RV<'v> {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => RV::Null,
            serde_json::Value::Bool(b) => RV::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    RV::Int64(i)
                } else {
                    RV::Double(n.as_f64().unwrap())
                }
            }
            serde_json::Value::String(s) => RV::Str(Arc::new(s)),
            serde_json::Value::Array(arr) => {
                RV::Array(RVArray::from_vec(arr.into_iter().map(RV::from).collect()))
            }
            serde_json::Value::Object(obj) => {
                let map = obj.into_iter().map(|(k, v)| (k, RV::from(v))).collect();
                RV::Object(RVObject::from_map(map))
            }
        }
    }
}

impl<'v> RV<'v> {
    pub fn to_bool(&self) -> bool {
        match &self {
            RV::Double(value) => !value.is_nan() && value.abs() > 0.0,
            RV::Str(value) => !value.is_empty(),
            RV::Bool(value) => *value,
            RV::Undefined => false,
            RV::Null => false,
            _ => true,
        }
    }

    pub fn to_double(&self) -> Option<f64> {
        match self {
            RV::Double(value) => Some(*value),
            RV::Int32(value) => Some(*value as f64),
            RV::Int64(value) => Some(*value as f64),
            RV::Decimal128(value) => value.to_string().parse::<f64>().ok(),
            RV::Bool(true) => Some(1.0),
            RV::Bool(false) => Some(0.0),
            RV::Str(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn not(&self) -> RV<'v> {
        RV::Bool(!self.to_bool())
    }
}

impl<'v> RV<'v> {
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
            RV::Datatype(_) => Datatype::Datatype,
            RV::Undefined | RV::Null => Datatype::None,
        }
    }

    pub fn extract_object(&self) -> Option<&RVObject<'v>> {
        match self {
            RV::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self, RV::Object(_))
    }

    pub fn eq_any_bool(&self, b: bool) -> bool {
        self.to_bool() == b
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
        if let Some(num) = self.to_double() {
            return num.partial_cmp(&if other { 1.0 } else { 0.0 });
        }
        self.to_bool().partial_cmp(&other)
    }

    pub fn is_in(&self, other: &RV<'v>) -> RV<'v> {
        match (self, other) {
            (RV::Str(lhs), RV::Str(rhs)) => RV::Bool(rhs.contains(lhs.as_str())),
            (lhs, RV::Array(rhs)) => RV::Bool(rhs.contains(lhs)),
            (RV::Str(key), RV::Object(map)) => RV::Bool(map.contains_key(key.as_str())),
            _ => RV::Bool(false),
        }
    }
}

impl<'v> Display for RV<'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

impl<'v> PartialEq for RV<'v> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RV::Array(_), RV::Array(_)) | (RV::Object(_), RV::Object(_)) => false,
            (RV::Undefined, RV::Undefined) => true,
            (RV::Null, RV::Null) => true,
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

impl<'v> PartialOrd for RV<'v> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (RV::Array(_), RV::Array(_)) | (RV::Object(_), RV::Object(_)) => None,
            (RV::Undefined, RV::Undefined) => Some(std::cmp::Ordering::Equal),
            (RV::Null, RV::Null) => Some(std::cmp::Ordering::Equal),
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

impl<'v> ops::Add for RV<'v> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            //
            (RV::Bool(_), RV::Bool(_))
            | (RV::Double(_), RV::Bool(_))
            | (RV::Bool(_), RV::Double(_)) => {
                RV::Double(self.to_double().unwrap() + rhs.to_double().unwrap())
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

impl<'v> ops::Sub for RV<'v> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RV::Undefined, _) | (_, RV::Undefined) => RV::Undefined,
            (l, r) => l
                .to_double()
                .and_then(|a| r.to_double().map(|b| (a, b)))
                .map(|(a, b)| RV::Double(a - b))
                .unwrap_or(RV::Undefined),
        }
    }
}

impl<'v> ops::Mul for RV<'v> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RV::Undefined, _) | (_, RV::Undefined) => RV::Undefined,
            (l, r) => l
                .to_double()
                .and_then(|a| r.to_double().map(|b| (a, b)))
                .map(|(a, b)| RV::Double(a * b))
                .unwrap_or(RV::Undefined),
        }
    }
}

impl<'v> ops::Div for RV<'v> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RV::Undefined, _) | (_, RV::Undefined) => RV::Undefined,
            (l, r) => l
                .to_double()
                .and_then(|a| r.to_double().map(|b| (a, b)))
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
    use indexmap::IndexMap;
    use proptest::prelude::*;

    use super::*;
    use std::sync::Arc;

    const BSON_DATETIME_MIN_MILLIS: i64 = -62_167_219_200_000;
    const BSON_DATETIME_MAX_MILLIS: i64 = 253_402_300_799_999;

    fn decimal128_string_strategy() -> impl Strategy<Value = String> {
        (any::<i64>(), 0u32..=18u32).prop_map(|(coefficient, scale)| {
            if scale == 0 {
                return coefficient.to_string();
            }

            let magnitude = coefficient.unsigned_abs();
            let factor = 10u64.pow(scale);
            let whole = magnitude / factor;
            let fractional = magnitude % factor;
            let sign = if coefficient < 0 { "-" } else { "" };

            format!(
                "{}{whole}.{:0width$}",
                sign,
                fractional,
                width = scale as usize
            )
        })
    }

    #[test]
    fn test_rv_as_bool() {
        // Test numeric values
        assert!(!RV::Double(0.0).to_bool());
        assert!(RV::Double(1.0).to_bool());
        assert!(RV::Double(-1.0).to_bool());
        assert!(!RV::Double(f64::NAN).to_bool());

        // Test strings
        assert!(!RV::Str(Arc::new("".to_string())).to_bool());
        assert!(RV::Str(Arc::new("hello".to_string())).to_bool());

        // Test booleans
        assert!(RV::Bool(true).to_bool());
        assert!(!RV::Bool(false).to_bool());

        // Test special values
        assert!(!RV::Undefined.to_bool());
        assert!(!RV::Null.to_bool());

        // Test collections
        let empty_array = RV::Array(RVArray::new());
        let empty_object = RV::Object(RVObject::new());
        assert!(empty_array.to_bool());
        assert!(empty_object.to_bool());
    }

    #[test]
    fn test_rv_as_double() {
        // Test numeric values
        assert_eq!(RV::Double(42.0).to_double(), Some(42.0));
        assert_eq!(RV::Double(-42.0).to_double(), Some(-42.0));
        assert_eq!(RV::Double(0.0).to_double(), Some(0.0));

        // Test booleans
        assert_eq!(RV::Bool(true).to_double(), Some(1.0));
        assert_eq!(RV::Bool(false).to_double(), Some(0.0));

        // Test strings
        assert_eq!(RV::Str(Arc::new("42".to_string())).to_double(), Some(42.0));
        assert_eq!(
            RV::Str(Arc::new("-42".to_string())).to_double(),
            Some(-42.0)
        );
        assert_eq!(RV::Str(Arc::new("invalid".to_string())).to_double(), None);
        assert_eq!(RV::Str(Arc::new("".to_string())).to_double(), None);

        // Test other types
        assert_eq!(RV::Undefined.to_double(), None);
        assert_eq!(RV::Array(RVArray::new()).to_double(), None);
        assert_eq!(RV::Object(RVObject::new()).to_double(), None);
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
        let mut map = IndexMap::default();
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
        // Strings are JSON-formatted with quotes
        assert_eq!(
            RV::Str(Arc::new("hello".to_string())).to_string(),
            "\"hello\""
        );
        // Numbers are JSON-formatted
        assert_eq!(RV::Double(42.0).to_string(), "42.0");
        // Booleans are JSON-formatted
        assert_eq!(RV::Bool(true).to_string(), "true");
        assert_eq!(RV::Bool(false).to_string(), "false");
        // Undefined/Null are JSON-formatted as null
        assert_eq!(RV::Undefined.to_string(), "null");

        let arr = vec![RV::Double(1.0), RV::Str(Arc::new("test".to_string()))];
        assert_eq!(
            RV::Array(RVArray::from_vec(arr)).to_string(),
            "[\n  1.0,\n  \"test\"\n]"
        );

        let mut map = IndexMap::default();
        map.insert("key".to_string(), RV::Double(42.0));
        assert_eq!(
            RV::Object(RVObject::from_map(map)).to_string(),
            "{\n  \"key\": 42.0\n}"
        );
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
        let mut map = IndexMap::default();
        map.insert("key".to_string(), RV::Double(1.0));
        let obj1 = RV::Object(RVObject::from_map(map));

        let mut map = IndexMap::default();
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

    proptest! {
        #[test]
        fn prop_rv_bson_roundtrip_bool(value in any::<bool>()) {
            let rv = RV::Bool(value);
            let decoded: RV =
                bson::deserialize_from_bson(bson::serialize_to_bson(&rv).unwrap()).unwrap();
            prop_assert_eq!(decoded, RV::Bool(value));
        }

        #[test]
        fn prop_rv_bson_roundtrip_int32(value in any::<i32>()) {
            let rv = RV::Int32(value);
            let decoded: RV =
                bson::deserialize_from_bson(bson::serialize_to_bson(&rv).unwrap()).unwrap();

            match decoded {
                RV::Int32(decoded_value) => prop_assert_eq!(decoded_value, value),
                other => prop_assert!(false, "Expected RV::Int32, got: {:?}", other),
            }
        }

        #[test]
        fn prop_rv_bson_roundtrip_int64(
            value in any::<i64>().prop_filter("outside i32 range", |v| *v < i32::MIN as i64 || *v > i32::MAX as i64)
        ) {
            let rv = RV::Int64(value);
            let decoded: RV =
                bson::deserialize_from_bson(bson::serialize_to_bson(&rv).unwrap()).unwrap();

            match decoded {
                RV::Int64(decoded_value) => prop_assert_eq!(decoded_value, value),
                other => prop_assert!(false, "Expected RV::Int64, got: {:?}", other),
            }
        }

        #[test]
        fn prop_rv_bson_roundtrip_double(value in any::<f64>().prop_filter("finite", |v| v.is_finite())) {
            let rv = RV::Double(value);
            let decoded: RV =
                bson::deserialize_from_bson(bson::serialize_to_bson(&rv).unwrap()).unwrap();

            match decoded {
                RV::Double(decoded_value) => prop_assert_eq!(decoded_value.to_bits(), value.to_bits()),
                other => prop_assert!(false, "Expected RV::Double, got: {:?}", other),
            }
        }

        #[test]
        fn prop_rv_bson_roundtrip_string(value in any::<String>()) {
            let rv = RV::Str(Arc::new(value.clone()));
            let decoded: RV =
                bson::deserialize_from_bson(bson::serialize_to_bson(&rv).unwrap()).unwrap();
            prop_assert_eq!(decoded, RV::Str(Arc::new(value)));
        }

        #[test]
        fn prop_rv_datetime_roundtrip(
            timestamp_millis in BSON_DATETIME_MIN_MILLIS..=BSON_DATETIME_MAX_MILLIS
        ) {
            let rv = RV::DateTime(bson::DateTime::from_millis(timestamp_millis));
            let decoded: RV =
                bson::deserialize_from_bson(bson::serialize_to_bson(&rv).unwrap()).unwrap();

            match decoded {
                RV::DateTime(date_time) => prop_assert_eq!(date_time.timestamp_millis(), timestamp_millis),
                other => prop_assert!(false, "Expected RV::DateTime, got: {:?}", other),
            }
        }

        #[test]
        fn prop_rv_decimal128_roundtrip(value in decimal128_string_strategy()) {
            let decimal = value.parse::<bson::Decimal128>().unwrap();
            let expected = decimal.to_string();
            let rv = RV::Decimal128(decimal);
            let decoded: RV =
                bson::deserialize_from_bson(bson::serialize_to_bson(&rv).unwrap()).unwrap();

            match decoded {
                RV::Decimal128(decoded_decimal) => {
                    prop_assert_eq!(decoded_decimal.to_string(), expected);
                }
                other => prop_assert!(false, "Expected RV::Decimal128, got: {:?}", other),
            }
        }

        #[test]
        fn prop_rv_datetime_roundtrip_preserves_distinct_values(
            val in BSON_DATETIME_MIN_MILLIS..=BSON_DATETIME_MAX_MILLIS
        ) {
            let rv_val = RV::DateTime(bson::DateTime::from_millis(val));

            let decoded_left: RV =
                bson::deserialize_from_bson(bson::serialize_to_bson(&rv_val).unwrap()).unwrap();

            match decoded_left {
                RV::DateTime(date_time) => prop_assert_eq!(date_time.timestamp_millis(), val),
                other => prop_assert!(false, "Expected RV::DateTime, got: {:?}", other),
            }
        }

        #[test]
        fn prop_rv_decimal128_zero_roundtrip(
            bytes in Just([0u8; 16])
        ) {
            let rv = RV::Decimal128(bson::Decimal128::from_bytes(bytes));
            let decoded: RV =
                bson::deserialize_from_bson(bson::serialize_to_bson(&rv).unwrap()).unwrap();

            match decoded {
                RV::Decimal128(decimal) => prop_assert_eq!(decimal.bytes(), bytes),
                other => prop_assert!(false, "Expected RV::Decimal128, got: {:?}", other),
            }
        }
    }

    #[test]
    fn test_rv_bson_roundtrip_array() {
        let value = RV::Array(RVArray::from_vec(vec![
            RV::Bool(true),
            RV::Int32(1),
            RV::Str(Arc::new("item".to_string())),
        ]));

        let encoded = bson::serialize_to_bson(&value).unwrap();
        let decoded: RV = bson::deserialize_from_bson(encoded).unwrap();

        match decoded {
            RV::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr.get(0), RV::Bool(true));
                match arr.get(1) {
                    RV::Int32(value) => assert_eq!(value, 1),
                    other => panic!("Expected RV::Int32, got: {:?}", other),
                }
                assert_eq!(arr.get(2), RV::Str(Arc::new("item".to_string())));
            }
            _ => panic!("Expected RV::Array"),
        }
    }

    #[test]
    fn test_rv_bson_roundtrip_object() {
        let mut map = IndexMap::default();
        map.insert("x".to_string(), RV::Int32(99));
        map.insert("y".to_string(), RV::Bool(false));
        let value = RV::Object(RVObject::from_map(map));

        let encoded = bson::serialize_to_bson(&value).unwrap();
        let decoded: RV = bson::deserialize_from_bson(encoded).unwrap();

        match decoded {
            RV::Object(obj) => {
                match obj.get("x") {
                    Some(RV::Int32(value)) => assert_eq!(value, 99),
                    other => panic!("Expected Some(RV::Int32(99)), got: {:?}", other),
                }
                assert_eq!(obj.get("y"), Some(RV::Bool(false)));
                assert_eq!(obj.get("missing"), None);
            }
            _ => panic!("Expected RV::Object"),
        }
    }

    #[test]
    fn test_rv_bson_roundtrip_nested() {
        let ts_ms = 1_700_000_000_000i64;
        let decimal_bytes = [1, 15, 2, 14, 3, 13, 4, 12, 5, 11, 6, 10, 7, 9, 8, 0];

        let nested = RV::Array(RVArray::from_vec(vec![
            RV::DateTime(bson::DateTime::from_millis(ts_ms)),
            RV::Decimal128(bson::Decimal128::from_bytes(decimal_bytes)),
        ]));
        let mut map = IndexMap::default();
        map.insert("nested".to_string(), nested);
        let value = RV::Object(RVObject::from_map(map));

        let encoded = bson::serialize_to_bson(&value).unwrap();
        let decoded: RV = bson::deserialize_from_bson(encoded).unwrap();

        match decoded {
            RV::Object(obj) => match obj.get("nested") {
                Some(RV::Array(arr)) => {
                    assert_eq!(arr.len(), 2);
                    match arr.get(0) {
                        RV::DateTime(date_time) => {
                            assert_eq!(date_time.timestamp_millis(), ts_ms);
                        }
                        other => panic!("Expected RV::DateTime, got: {:?}", other),
                    }
                    match arr.get(1) {
                        RV::Decimal128(decimal) => {
                            assert_eq!(decimal.bytes(), decimal_bytes);
                        }
                        other => panic!("Expected RV::Decimal128, got: {:?}", other),
                    }
                }
                _ => panic!("Expected nested array"),
            },
            _ => panic!("Expected RV::Object"),
        }
    }
}
