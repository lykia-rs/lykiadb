pub mod callable;
pub mod environment;
pub mod eval;

pub mod std_val;
pub use std_val::*;

use lykiadb_lang::types::Datatype;
use std::fmt::{Debug, Display};
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
use serde::{Serialize, Deserialize};

pub trait Value: 
    Clone + 
    Debug + 
    Display + 
    PartialEq + 
    PartialOrd +
    Add<Output = Self> +
    Sub<Output = Self> +
    Mul<Output = Self> +
    Div<Output = Self> +
    Serialize +
    for<'de> Deserialize<'de> +
    Send + 
    Sync +
    Sized
{
    /// Associated type for storing arrays of values
    type Array: Clone + Debug;
    
    /// Associated type for storing object/map values
    type Object: Clone + Debug;
    
    /// Associated type for callable functions
    type Callable: Clone + Debug;
    
    /// Create a string value
    fn string(s: String) -> Self;
    
    /// Create a numeric value
    fn number(n: f64) -> Self;
    
    /// Create a boolean value
    fn boolean(b: bool) -> Self;
    
    /// Create an array value
    fn array(arr: Self::Array) -> Self;
    
    /// Create an object value
    fn object(obj: Self::Object) -> Self;
    
    /// Create a callable value
    fn callable(c: Self::Callable) -> Self;
    
    /// Create an undefined/null value
    fn undefined() -> Self;
    
    /// Get the datatype of this value
    fn get_type(&self) -> Datatype;
    
    /// Convert to boolean (truthiness)
    fn as_bool(&self) -> bool;
    
    /// Convert to number if possible
    fn as_number(&self) -> Option<f64>;
    
    /// Convert to string representation
    fn as_string(&self) -> Option<String>;
    
    /// Logical NOT operation
    fn not(&self) -> Self {
        Self::boolean(!self.as_bool())
    }
    
    /// Check if this value is contained in another
    fn is_in(&self, other: &Self) -> Self;
    
    /// Check equality with any boolean (for coercion)
    fn eq_any_bool(&self, b: bool) -> bool {
        self.as_bool() == b
    }
    
    /// Check string-number equality (for coercion)
    fn eq_str_num(&self, n: f64) -> bool;
    
    /// Compare string with boolean (for mixed type comparisons)
    fn partial_cmp_str_bool(&self, other: bool) -> Option<Ordering>;

}

/// Trait for array operations
pub trait ValueArray<V: Value>: Clone + Debug {
    /// Get the length of the array
    fn len(&self) -> usize;
    
    /// Check if array is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Get element at index
    fn get(&self, index: usize) -> Option<&V>;
    
    /// Check if array contains a value
    fn contains(&self, value: &V) -> bool;
    
    /// Iterate over array elements
    fn iter(&self) -> Box<dyn Iterator<Item = &V> + '_>;
}

/// Trait for object operations
pub trait ValueObject<V: Value>: Clone + Debug {
    /// Get the number of key-value pairs
    fn len(&self) -> usize;
    
    /// Check if object is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Get value by key
    fn get(&self, key: &str) -> Option<&V>;
    
    /// Check if object contains a key
    fn contains_key(&self, key: &str) -> bool;
    
    /// Iterate over keys
    fn keys(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    
    /// Iterate over key-value pairs
    fn iter(&self) -> Box<dyn Iterator<Item = (&str, &V)> + '_>;
}

/// Trait for callable operations
pub trait ValueCallable: Clone + Debug {
    /// Get parameter types
    fn parameter_types(&self) -> &Datatype;
    
    /// Get return type
    fn return_type(&self) -> &Datatype;
}