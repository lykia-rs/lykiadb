use lykiadb_lang::types::Datatype;
use std::fmt::{Debug, Display};
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
use rustc_hash::FxHashMap;
use serde::{Serialize, Deserialize};

/// Zero-cost abstraction trait for value types in LykiaDB.
/// 
/// This trait enables compile-time polymorphism between different value 
/// implementations (RV enum, BSON, etc.) without runtime overhead.
/// All methods are designed to be inlineable for zero-cost abstraction.
pub trait ValueType: 
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

    // ==================== Constructors ====================
    
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
    
    /// Create a datatype value
    fn datatype(dt: Datatype) -> Self;
    
    /// Create an undefined/null value
    fn undefined() -> Self;

    // ==================== Type Checking ====================
    
    /// Get the datatype of this value
    fn get_type(&self) -> Datatype;
    
    /// Check if value is a string
    #[inline]
    fn is_string(&self) -> bool {
        matches!(self.get_type(), Datatype::Str)
    }
    
    /// Check if value is a number
    #[inline]
    fn is_number(&self) -> bool {
        matches!(self.get_type(), Datatype::Num)
    }
    
    /// Check if value is a boolean
    #[inline]
    fn is_boolean(&self) -> bool {
        matches!(self.get_type(), Datatype::Bool)
    }
    
    /// Check if value is an array
    #[inline]
    fn is_array(&self) -> bool {
        matches!(self.get_type(), Datatype::Array(_))
    }
    
    /// Check if value is an object
    #[inline]
    fn is_object(&self) -> bool {
        matches!(self.get_type(), Datatype::Object(_))
    }
    
    /// Check if value is undefined/null
    #[inline]
    fn is_undefined(&self) -> bool {
        matches!(self.get_type(), Datatype::None)
    }

    // ==================== Conversions ====================
    
    /// Convert to boolean (truthiness)
    fn as_bool(&self) -> bool;
    
    /// Convert to number if possible
    fn as_number(&self) -> Option<f64>;
    
    /// Convert to string representation
    fn as_string(&self) -> Option<String>;

    // ==================== Operations ====================
    
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

    // ==================== Access Methods ====================
    
    /// Get array elements (if this is an array)
    fn as_array(&self) -> Option<&Self::Array>;
    
    /// Get object data (if this is an object)  
    fn as_object(&self) -> Option<&Self::Object>;
    
    /// Get callable data (if this is a callable)
    fn as_callable(&self) -> Option<&Self::Callable>;
}

/// Trait for array operations
pub trait ValueArray<V: ValueType>: Clone + Debug {
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
pub trait ValueObject<V: ValueType>: Clone + Debug {
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

// ==================== Binary Operations ====================

/// Enum for binary operations (from your existing code)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    IsEqual,
    IsNotEqual,
    Is,
    IsNot,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    In,
    NotIn,
}

/// Generic binary evaluation function that works with any ValueType
#[inline]
pub fn eval_binary<V: ValueType>(left: V, right: V, operation: Operation) -> V {
    match operation {
        Operation::Add => left + right,
        Operation::Subtract => left - right,
        Operation::Multiply => left * right,
        Operation::Divide => left / right,
        Operation::IsEqual => V::boolean(left == right),
        Operation::IsNotEqual => V::boolean(left != right),
        Operation::Less => V::boolean(left < right),
        Operation::LessEqual => V::boolean(left <= right),
        Operation::Greater => V::boolean(left > right),
        Operation::GreaterEqual => V::boolean(left >= right),
        Operation::And => V::boolean(left.as_bool() && right.as_bool()),
        Operation::Or => V::boolean(left.as_bool() || right.as_bool()),
        Operation::In => left.is_in(&right),
        Operation::NotIn => left.is_in(&right).not(),
        // Add other operations as needed
        _ => V::undefined(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test that the trait compiles and can be used generically
    fn test_generic_operations<V: ValueType>() {
        let a = V::number(42.0);
        let b = V::number(24.0);
        
        // Test arithmetic
        let sum = a.clone() + b.clone();
        let diff = a.clone() - b.clone();
        let prod = a.clone() * b.clone();
        let quot = a.clone() / b.clone();
        
        // Test comparisons
        let eq = eval_binary(a.clone(), b.clone(), Operation::IsEqual);
        let lt = eval_binary(a.clone(), b.clone(), Operation::Less);
        
        // Test conversions
        assert!(a.as_bool());
        assert_eq!(a.as_number(), Some(42.0));
        
        // Test type checking
        assert!(a.is_number());
        assert!(!a.is_string());
    }
}