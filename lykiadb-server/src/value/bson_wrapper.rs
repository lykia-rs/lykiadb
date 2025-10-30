use super::traits::{ValueType, ValueArray, ValueObject, ValueCallable};
use lykiadb_lang::types::Datatype;
use rustc_hash::FxHashMap;
use std::fmt::{Debug, Display, Formatter};
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use bson::{Bson, Document};

/// BSON-based value implementation that implements ValueType
/// This provides a BSON-native alternative to RV with the same interface
#[derive(Debug, Clone, PartialEq)]
pub struct BsonValue(pub Bson);

/// Array wrapper for BSON
#[derive(Debug, Clone)]
pub struct BsonArray(Vec<Bson>);

/// Object wrapper for BSON  
#[derive(Debug, Clone)]
pub struct BsonObject(Document);

/// Callable wrapper for BSON (stores metadata as a document)
#[derive(Debug, Clone)]
pub struct BsonCallable {
    parameter_types: Datatype,
    return_type: Datatype,
    // In a real implementation, you might store the function pointer differently
    // For now, we'll just store the type information
}

// ==================== BsonArray Implementation ====================

impl ValueArray<BsonValue> for BsonArray {
    fn len(&self) -> usize {
        self.0.len()
    }
    
    fn get(&self, index: usize) -> Option<&BsonValue> {
        // We need to convert &Bson to &BsonValue, which requires unsafe casting
        // or redesigning. For safety, let's return None for now
        None // TODO: Safe conversion from &Bson to &BsonValue
    }
    
    fn contains(&self, value: &BsonValue) -> bool {
        self.0.contains(&value.0)
    }
    
    fn iter(&self) -> Box<dyn Iterator<Item = &BsonValue> + '_> {
        // Same issue as above - we can't safely convert &Bson to &BsonValue
        // without unsafe code or redesigning the API
        todo!("Safe iterator conversion needed")
    }
}

impl BsonArray {
    pub fn new(values: Vec<BsonValue>) -> Self {
        BsonArray(values.into_iter().map(|v| v.0).collect())
    }
    
    pub fn from_bson_vec(bsons: Vec<Bson>) -> Self {
        BsonArray(bsons)
    }
    
    pub fn contains_bson(&self, value: &Bson) -> bool {
        self.0.contains(value)
    }
}

// ==================== BsonObject Implementation ====================

impl ValueObject<BsonValue> for BsonObject {
    fn len(&self) -> usize {
        self.0.len()
    }
    
    fn get(&self, key: &str) -> Option<&BsonValue> {
        // Same conversion issue as with arrays
        None // TODO: Safe conversion from &Bson to &BsonValue
    }
    
    fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
    
    fn keys(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(self.0.keys().map(|s| s.as_str()))
    }
    
    fn iter(&self) -> Box<dyn Iterator<Item = (&str, &BsonValue)> + '_> {
        // Same conversion issue
        todo!("Safe iterator conversion needed")
    }
}

impl BsonObject {
    pub fn new() -> Self {
        BsonObject(Document::new())
    }
    
    pub fn from_document(doc: Document) -> Self {
        BsonObject(doc)
    }
    
    pub fn get_bson(&self, key: &str) -> Option<&Bson> {
        self.0.get(key)
    }
    
    pub fn insert_bson(&mut self, key: String, value: Bson) {
        self.0.insert(key, value);
    }
}

// ==================== BsonCallable Implementation ====================

impl ValueCallable for BsonCallable {
    fn parameter_types(&self) -> &Datatype {
        &self.parameter_types
    }
    
    fn return_type(&self) -> &Datatype {
        &self.return_type
    }
}

impl BsonCallable {
    pub fn new(parameter_types: Datatype, return_type: Datatype) -> Self {
        BsonCallable {
            parameter_types,
            return_type,
        }
    }
}

// ==================== Core ValueType Implementation ====================

impl ValueType for BsonValue {
    type Array = BsonArray;
    type Object = BsonObject;
    type Callable = BsonCallable;

    // ==================== Constructors ====================
    
    fn string(s: String) -> Self {
        BsonValue(Bson::String(s))
    }
    
    fn number(n: f64) -> Self {
        BsonValue(Bson::Double(n))
    }
    
    fn boolean(b: bool) -> Self {
        BsonValue(Bson::Boolean(b))
    }
    
    fn array(arr: Self::Array) -> Self {
        BsonValue(Bson::Array(arr.0))
    }
    
    fn object(obj: Self::Object) -> Self {
        BsonValue(Bson::Document(obj.0))
    }
    
    fn callable(c: Self::Callable) -> Self {
        // Store callable metadata as a document
        let mut doc = Document::new();
        // In a real implementation, you'd serialize the callable info properly
        doc.insert("__type", "callable");
        doc.insert("param_types", format!("{:?}", c.parameter_types));
        doc.insert("return_type", format!("{:?}", c.return_type));
        BsonValue(Bson::Document(doc))
    }
    
    fn datatype(dt: Datatype) -> Self {
        let mut doc = Document::new();
        doc.insert("__type", "datatype");
        doc.insert("datatype", format!("{:?}", dt));
        BsonValue(Bson::Document(doc))
    }
    
    fn undefined() -> Self {
        BsonValue(Bson::Null)
    }

    // ==================== Type Operations ====================
    
    fn get_type(&self) -> Datatype {
        match &self.0 {
            Bson::String(_) => Datatype::Str,
            Bson::Double(_) | Bson::Int32(_) | Bson::Int64(_) => Datatype::Num,
            Bson::Boolean(_) => Datatype::Bool,
            Bson::Array(arr) => {
                if arr.is_empty() {
                    Datatype::Array(Box::new(Datatype::None))
                } else {
                    let first_type = BsonValue(arr[0].clone()).get_type();
                    Datatype::Array(Box::new(first_type))
                }
            }
            Bson::Document(doc) => {
                if let Some(Bson::String(type_str)) = doc.get("__type") {
                    match type_str.as_str() {
                        "callable" => {
                            // Parse callable types - simplified for now
                            Datatype::Callable(Box::new(Datatype::Unknown), Box::new(Datatype::Unknown))
                        }
                        "datatype" => Datatype::Datatype,
                        _ => {
                            // Regular object
                            if doc.is_empty() {
                                Datatype::None
                            } else {
                                let mut object_types = FxHashMap::default();
                                for (key, value) in doc {
                                    let value_type = BsonValue(value.clone()).get_type();
                                    object_types.insert(key.clone(), value_type);
                                }
                                Datatype::Object(object_types)
                            }
                        }
                    }
                } else {
                    // Regular object
                    if doc.is_empty() {
                        Datatype::None
                    } else {
                        let mut object_types = FxHashMap::default();
                        for (key, value) in doc {
                            let value_type = BsonValue(value.clone()).get_type();
                            object_types.insert(key.clone(), value_type);
                        }
                        Datatype::Object(object_types)
                    }
                }
            }
            Bson::Null => Datatype::None,
            _ => Datatype::Unknown,
        }
    }
    
    fn as_bool(&self) -> bool {
        match &self.0 {
            Bson::Boolean(b) => *b,
            Bson::Double(n) => !n.is_nan() && n.abs() > 0.0,
            Bson::Int32(n) => *n != 0,
            Bson::Int64(n) => *n != 0,
            Bson::String(s) => !s.is_empty(),
            Bson::Null => false,
            _ => true,
        }
    }
    
    fn as_number(&self) -> Option<f64> {
        match &self.0 {
            Bson::Double(n) => Some(*n),
            Bson::Int32(n) => Some(*n as f64),
            Bson::Int64(n) => Some(*n as f64),
            Bson::Boolean(true) => Some(1.0),
            Bson::Boolean(false) => Some(0.0),
            Bson::String(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }
    
    fn as_string(&self) -> Option<String> {
        match &self.0 {
            Bson::String(s) => Some(s.clone()),
            _ => None,
        }
    }
    
    fn is_in(&self, other: &Self) -> Self {
        match (&self.0, &other.0) {
            (Bson::String(needle), Bson::String(haystack)) => {
                Self::boolean(haystack.contains(needle))
            }
            (needle, Bson::Array(haystack)) => {
                Self::boolean(haystack.contains(needle))
            }
            (Bson::String(key), Bson::Document(doc)) => {
                Self::boolean(doc.contains_key(key))
            }
            _ => Self::boolean(false),
        }
    }
    
    fn eq_str_num(&self, n: f64) -> bool {
        if let Bson::String(s) = &self.0 {
            if let Ok(num) = s.parse::<f64>() {
                return num == n;
            }
        }
        false
    }
    
    fn partial_cmp_str_bool(&self, other: bool) -> Option<Ordering> {
        if let Some(num) = self.as_number() {
            return num.partial_cmp(&if other { 1.0 } else { 0.0 });
        }
        self.as_bool().partial_cmp(&other)
    }
    
    // ==================== Access Methods ====================
    
    fn as_array(&self) -> Option<&Self::Array> {
        // Lifetime/ownership issues - would need redesign
        None
    }
    
    fn as_object(&self) -> Option<&Self::Object> {
        // Lifetime/ownership issues - would need redesign
        None
    }
    
    fn as_callable(&self) -> Option<&Self::Callable> {
        // Lifetime/ownership issues - would need redesign
        None
    }
}

// ==================== Trait Implementations ====================

impl PartialOrd for BsonValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.0, &other.0) {
            // Same types
            (Bson::String(a), Bson::String(b)) => Some(a.cmp(b)),
            (Bson::Double(a), Bson::Double(b)) => a.partial_cmp(b),
            (Bson::Int32(a), Bson::Int32(b)) => a.partial_cmp(b),
            (Bson::Int64(a), Bson::Int64(b)) => a.partial_cmp(b),
            (Bson::Boolean(a), Bson::Boolean(b)) => a.partial_cmp(b),
            
            // Numeric conversions
            (Bson::Double(a), Bson::Int32(b)) => a.partial_cmp(&(*b as f64)),
            (Bson::Double(a), Bson::Int64(b)) => a.partial_cmp(&(*b as f64)),
            (Bson::Int32(a), Bson::Double(b)) => (*a as f64).partial_cmp(b),
            (Bson::Int64(a), Bson::Double(b)) => (*a as f64).partial_cmp(b),
            (Bson::Int32(a), Bson::Int64(b)) => (*a as i64).partial_cmp(b),
            (Bson::Int64(a), Bson::Int32(b)) => a.partial_cmp(&(*b as i64)),
            
            // Cross-type comparisons (string-number, etc.)
            (Bson::String(a), Bson::Double(b)) => {
                if let Ok(num) = a.parse::<f64>() {
                    num.partial_cmp(b)
                } else {
                    None
                }
            }
            (Bson::Double(a), Bson::String(b)) => {
                if let Ok(num) = b.parse::<f64>() {
                    a.partial_cmp(&num)
                } else {
                    None
                }
            }
            
            // Null comparisons
            (Bson::Null, Bson::Null) => Some(Ordering::Equal),
            (Bson::Null, _) | (_, Bson::Null) => None,
            
            // Arrays and documents don't have ordering
            (Bson::Array(_), Bson::Array(_)) | 
            (Bson::Document(_), Bson::Document(_)) => None,
            
            _ => None,
        }
    }
}

impl Display for BsonValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Bson::String(s) => write!(f, "{}", s),
            Bson::Double(n) => write!(f, "{}", n),
            Bson::Int32(n) => write!(f, "{}", n),
            Bson::Int64(n) => write!(f, "{}", n),
            Bson::Boolean(b) => write!(f, "{}", b),
            Bson::Null => write!(f, "undefined"),
            Bson::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", BsonValue(item.clone()))?;
                }
                write!(f, "]")
            }
            Bson::Document(doc) => {
                // Check if it's a special type
                if let Some(Bson::String(type_str)) = doc.get("__type") {
                    match type_str.as_str() {
                        "callable" => write!(f, "<Callable>"),
                        "datatype" => {
                            if let Some(Bson::String(dt_str)) = doc.get("datatype") {
                                write!(f, "<Datatype, {}>", dt_str)
                            } else {
                                write!(f, "<Datatype>")
                            }
                        }
                        _ => {
                            // Regular object
                            write!(f, "{{")?;
                            for (i, (key, value)) in doc.iter().enumerate() {
                                if i != 0 {
                                    write!(f, ", ")?;
                                }
                                write!(f, "{}: {}", key, BsonValue(value.clone()))?;
                            }
                            write!(f, "}}")
                        }
                    }
                } else {
                    // Regular object
                    write!(f, "{{")?;
                    for (i, (key, value)) in doc.iter().enumerate() {
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}: {}", key, BsonValue(value.clone()))?;
                    }
                    write!(f, "}}")
                }
            }
            _ => write!(f, "{:?}", self.0),
        }
    }
}

impl Add for BsonValue {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        if let (Some(a), Some(b)) = (self.as_number(), rhs.as_number()) {
            Self::number(a + b)
        } else {
            Self::undefined()
        }
    }
}

impl Sub for BsonValue {
    type Output = Self;
    
    fn sub(self, rhs: Self) -> Self::Output {
        if let (Some(a), Some(b)) = (self.as_number(), rhs.as_number()) {
            Self::number(a - b)
        } else {
            Self::undefined()
        }
    }
}

impl Mul for BsonValue {
    type Output = Self;
    
    fn mul(self, rhs: Self) -> Self::Output {
        if let (Some(a), Some(b)) = (self.as_number(), rhs.as_number()) {
            Self::number(a * b)
        } else {
            Self::undefined()
        }
    }
}

impl Div for BsonValue {
    type Output = Self;
    
    fn div(self, rhs: Self) -> Self::Output {
        if let (Some(a), Some(b)) = (self.as_number(), rhs.as_number()) {
            if a == 0.0 && b == 0.0 {
                Self::undefined()
            } else {
                Self::number(a / b)
            }
        } else {
            Self::undefined()
        }
    }
}

impl Serialize for BsonValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Bson::deserialize(deserializer).map(BsonValue)
    }
}

// ==================== Conversion Helpers ====================

impl From<Bson> for BsonValue {
    fn from(bson: Bson) -> Self {
        BsonValue(bson)
    }
}

impl From<BsonValue> for Bson {
    fn from(bson_value: BsonValue) -> Self {
        bson_value.0
    }
}

impl AsRef<Bson> for BsonValue {
    fn as_ref(&self) -> &Bson {
        &self.0
    }
}

impl AsMut<Bson> for BsonValue {
    fn as_mut(&mut self) -> &mut Bson {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::traits::eval_binary;
    use crate::value::traits::Operation;
    
    #[test]
    fn test_bson_value_basic_operations() {
        let a = BsonValue::number(42.0);
        let b = BsonValue::number(24.0);
        
        // Test arithmetic
        let sum = a.clone() + b.clone();
        assert_eq!(sum.as_number(), Some(66.0));
        
        let diff = a.clone() - b.clone();
        assert_eq!(diff.as_number(), Some(18.0));
        
        // Test comparisons
        let eq = eval_binary(a.clone(), b.clone(), Operation::IsEqual);
        assert_eq!(eq.as_bool(), false);
        
        let lt = eval_binary(b.clone(), a.clone(), Operation::Less);
        assert_eq!(lt.as_bool(), true);
    }
    
    #[test]
    fn test_bson_value_conversions() {
        let bson = Bson::Double(42.0);
        let bson_value = BsonValue::from(bson.clone());
        let back_to_bson: Bson = bson_value.into();
        
        assert_eq!(bson, back_to_bson);
    }
    
    #[test]
    fn test_bson_value_type_checking() {
        let num = BsonValue::number(42.0);
        assert!(num.is_number());
        assert!(!num.is_string());
        assert!(!num.is_boolean());
        
        let str_val = BsonValue::string("hello".to_string());
        assert!(str_val.is_string());
        assert!(!str_val.is_number());
    }
    
    #[test]
    fn test_bson_value_display() {
        assert_eq!(BsonValue::string("hello".to_string()).to_string(), "hello");
        assert_eq!(BsonValue::number(42.0).to_string(), "42");
        assert_eq!(BsonValue::boolean(true).to_string(), "true");
        assert_eq!(BsonValue::boolean(false).to_string(), "false");
        assert_eq!(BsonValue::undefined().to_string(), "undefined");
    }
}