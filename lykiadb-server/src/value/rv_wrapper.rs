use super::{RV, callable::Callable};
use super::traits::{ValueType, ValueArray, ValueObject, ValueCallable};
use lykiadb_lang::types::Datatype;
use rustc_hash::FxHashMap;
use std::sync::{Arc, RwLock};
use std::fmt::{Debug, Display, Formatter};
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use crate::util::{Shared, alloc_shared};

/// Zero-cost wrapper around the existing RV enum that implements ValueType
/// This maintains complete backward compatibility while enabling trait abstraction
#[derive(Debug, Clone)]
pub struct RvValue(pub RV);

/// Array wrapper for RV
#[derive(Debug, Clone)]
pub struct RvArray(Shared<Vec<RV>>);

/// Object wrapper for RV
#[derive(Debug, Clone)]
pub struct RvObject(Shared<FxHashMap<String, RV>>);

/// Callable wrapper for RV
#[derive(Debug, Clone)]
pub struct RvCallable(Callable);

// ==================== RvArray Implementation ====================

impl ValueArray<RvValue> for RvArray {
    fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }
    
    fn get(&self, index: usize) -> Option<&RvValue> {
        // Note: This is a bit tricky because we can't return a reference to the interior
        // of a RwLock. In practice, you might need to redesign this or use Cow/Arc
        // For now, let's return None and handle this differently
        None // TODO: Redesign to work with locks
    }
    
    fn contains(&self, value: &RvValue) -> bool {
        self.0.read().unwrap().contains(&value.0)
    }
    
    fn iter(&self) -> Box<dyn Iterator<Item = &RvValue> + '_> {
        // This also has lifetime issues with RwLock
        // In practice, you'd need a different approach, perhaps returning owned values
        // or using a different container type
        todo!("Iterator with RwLock requires careful lifetime management")
    }
}

// ==================== RvObject Implementation ====================

impl ValueObject<RvValue> for RvObject {
    fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }
    
    fn get(&self, key: &str) -> Option<&RvValue> {
        // Same lifetime issue as with arrays
        None // TODO: Redesign to work with locks
    }
    
    fn contains_key(&self, key: &str) -> bool {
        self.0.read().unwrap().contains_key(key)
    }
    
    fn keys(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        todo!("Iterator with RwLock requires careful lifetime management")
    }
    
    fn iter(&self) -> Box<dyn Iterator<Item = (&str, &RvValue)> + '_> {
        todo!("Iterator with RwLock requires careful lifetime management")
    }
}

// ==================== RvCallable Implementation ====================

impl ValueCallable for RvCallable {
    fn parameter_types(&self) -> &Datatype {
        &self.0.parameter_types
    }
    
    fn return_type(&self) -> &Datatype {
        &self.0.return_type
    }
}

// ==================== Core ValueType Implementation ====================

impl ValueType for RvValue {
    type Array = RvArray;
    type Object = RvObject;
    type Callable = RvCallable;

    // ==================== Constructors ====================
    
    fn string(s: String) -> Self {
        RvValue(RV::Str(Arc::new(s)))
    }
    
    fn number(n: f64) -> Self {
        RvValue(RV::Num(n))
    }
    
    fn boolean(b: bool) -> Self {
        RvValue(RV::Bool(b))
    }
    
    fn array(arr: Self::Array) -> Self {
        RvValue(RV::Array(arr.0))
    }
    
    fn object(obj: Self::Object) -> Self {
        RvValue(RV::Object(obj.0))
    }
    
    fn callable(c: Self::Callable) -> Self {
        RvValue(RV::Callable(c.0))
    }
    
    fn datatype(dt: Datatype) -> Self {
        RvValue(RV::Datatype(dt))
    }
    
    fn undefined() -> Self {
        RvValue(RV::Undefined)
    }

    // ==================== Type Operations ====================
    
    fn get_type(&self) -> Datatype {
        self.0.get_type()
    }
    
    fn as_bool(&self) -> bool {
        self.0.as_bool()
    }
    
    fn as_number(&self) -> Option<f64> {
        self.0.as_number()
    }
    
    fn as_string(&self) -> Option<String> {
        match &self.0 {
            RV::Str(s) => Some(s.as_ref().clone()),
            _ => None,
        }
    }
    
    fn is_in(&self, other: &Self) -> Self {
        RvValue(self.0.is_in(&other.0))
    }
    
    fn eq_str_num(&self, n: f64) -> bool {
        self.0.eq_str_num(n)
    }
    
    fn partial_cmp_str_bool(&self, other: bool) -> Option<Ordering> {
        self.0.partial_cmp_str_bool(other)
    }
    
    // ==================== Access Methods ====================
    
    fn as_array(&self) -> Option<&Self::Array> {
        // Lifetime issues with the current design
        None // TODO: Redesign
    }
    
    fn as_object(&self) -> Option<&Self::Object> {
        // Lifetime issues with the current design
        None // TODO: Redesign
    }
    
    fn as_callable(&self) -> Option<&Self::Callable> {
        // Lifetime issues with the current design
        None // TODO: Redesign
    }
}

// ==================== Trait Implementations ====================

impl PartialEq for RvValue {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for RvValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Display for RvValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for RvValue {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        RvValue(self.0 + rhs.0)
    }
}

impl Sub for RvValue {
    type Output = Self;
    
    fn sub(self, rhs: Self) -> Self::Output {
        RvValue(self.0 - rhs.0)
    }
}

impl Mul for RvValue {
    type Output = Self;
    
    fn mul(self, rhs: Self) -> Self::Output {
        RvValue(self.0 * rhs.0)
    }
}

impl Div for RvValue {
    type Output = Self;
    
    fn div(self, rhs: Self) -> Self::Output {
        RvValue(self.0 / rhs.0)
    }
}

impl Serialize for RvValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RvValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RV::deserialize(deserializer).map(RvValue)
    }
}

// ==================== Conversion Helpers ====================

impl From<RV> for RvValue {
    fn from(rv: RV) -> Self {
        RvValue(rv)
    }
}

impl From<RvValue> for RV {
    fn from(rv_value: RvValue) -> Self {
        rv_value.0
    }
}

impl AsRef<RV> for RvValue {
    fn as_ref(&self) -> &RV {
        &self.0
    }
}

impl AsMut<RV> for RvValue {
    fn as_mut(&mut self) -> &mut RV {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::traits::eval_binary;
    use crate::value::traits::Operation;
    
    #[test]
    fn test_rv_value_basic_operations() {
        let a = RvValue::number(42.0);
        let b = RvValue::number(24.0);
        
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
    fn test_rv_value_conversions() {
        let rv = RV::Num(42.0);
        let rv_value = RvValue::from(rv.clone());
        let back_to_rv: RV = rv_value.into();
        
        assert_eq!(rv, back_to_rv);
    }
    
    #[test]
    fn test_rv_value_type_checking() {
        let num = RvValue::number(42.0);
        assert!(num.is_number());
        assert!(!num.is_string());
        assert!(!num.is_boolean());
        
        let str_val = RvValue::string("hello".to_string());
        assert!(str_val.is_string());
        assert!(!str_val.is_number());
    }
}