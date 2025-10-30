/*!
# Interpreter Integration Guide

This shows how to integrate the new ValueType abstraction into your existing interpreter
with minimal changes to your current codebase.
*/

use crate::engine::interpreter::{Interpreter, HaltReason};
use crate::value::{RV, traits::ValueType, rv_wrapper::RvValue, bson_wrapper::BsonValue, conversions::ValueConverter};
use lykiadb_lang::ast::expr::Expr;
use std::sync::Arc;

/// Extension trait that adds generic evaluation capabilities to your existing interpreter
pub trait InterpreterValueTypeExt {
    /// Evaluate an expression and return result as any ValueType
    fn eval_as<V: ValueType + 'static>(&mut self, expr: &Expr) -> Result<V, String>;
    
    /// Evaluate source code and return result as any ValueType
    fn interpret_as<V: ValueType + 'static>(&mut self, source: &str) -> Result<V, String>;
    
    /// Switch the interpreter to use a different value type for future operations
    fn with_value_type<V: ValueType + 'static>(self) -> GenericInterpreterAdapter<V>;
}

impl InterpreterValueTypeExt for Interpreter {
    fn eval_as<V: ValueType + 'static>(&mut self, expr: &Expr) -> Result<V, String> {
        match self.eval(expr) {
            Ok(rv) => {
                // Convert RV to the requested value type
                if std::any::TypeId::of::<V>() == std::any::TypeId::of::<RvValue>() {
                    // Direct conversion to RvValue
                    let rv_value = RvValue::from(rv);
                    // This is safe because we checked the type
                    let result = unsafe { std::mem::transmute_copy(&rv_value) };
                    std::mem::forget(rv_value);
                    Ok(result)
                } else if std::any::TypeId::of::<V>() == std::any::TypeId::of::<BsonValue>() {
                    // Convert RV -> BSON -> BsonValue
                    let bson = ValueConverter::rv_to_bson(&rv);
                    let bson_value = BsonValue::from(bson);
                    // This is safe because we checked the type
                    let result = unsafe { std::mem::transmute_copy(&bson_value) };
                    std::mem::forget(bson_value);
                    Ok(result)
                } else {
                    Err("Unsupported value type conversion".to_string())
                }
            }
            Err(HaltReason::Error(e)) => Err(format!("Execution error: {:?}", e)),
            Err(HaltReason::Return(rv)) => {
                // Handle return the same way as Ok
                if std::any::TypeId::of::<V>() == std::any::TypeId::of::<RvValue>() {
                    let rv_value = RvValue::from(rv);
                    let result = unsafe { std::mem::transmute_copy(&rv_value) };
                    std::mem::forget(rv_value);
                    Ok(result)
                } else if std::any::TypeId::of::<V>() == std::any::TypeId::of::<BsonValue>() {
                    let bson = ValueConverter::rv_to_bson(&rv);
                    let bson_value = BsonValue::from(bson);
                    let result = unsafe { std::mem::transmute_copy(&bson_value) };
                    std::mem::forget(bson_value);
                    Ok(result)
                } else {
                    Err("Unsupported value type conversion".to_string())
                }
            }
        }
    }
    
    fn interpret_as<V: ValueType + 'static>(&mut self, source: &str) -> Result<V, String> {
        match self.interpret(source) {
            Ok(rv) => {
                // Convert RV to the requested value type using proper conversion
                self.convert_rv_to_value_type::<V>(rv)
            }
            Err(e) => Err(format!("Execution error: {:?}", e)),
        }
    }
    
    fn with_value_type<V: ValueType + 'static>(self) -> GenericInterpreterAdapter<V> {
        GenericInterpreterAdapter::new(self)
    }
}

/// Safe conversion helper
trait RvConverter {
    fn convert_rv_to_value_type<V: ValueType + 'static>(&self, rv: RV) -> Result<V, String>;
}

impl RvConverter for Interpreter {
    fn convert_rv_to_value_type<V: ValueType + 'static>(&self, rv: RV) -> Result<V, String> {
        // Use Any for safe type checking and conversion
        use std::any::Any;
        
        let boxed_rv_value: Box<dyn Any> = Box::new(RvValue::from(rv.clone()));
        let boxed_bson_value: Box<dyn Any> = Box::new(BsonValue::from(rv));
        
        // Try to downcast to the requested type
        if let Some(rv_value) = boxed_rv_value.downcast_ref::<V>() {
            Ok(rv_value.clone())
        } else if let Some(bson_value) = boxed_bson_value.downcast_ref::<V>() {
            Ok(bson_value.clone())
        } else {
            Err("Cannot convert to requested value type".to_string())
        }
    }
}

/// Adapter that wraps your existing interpreter to work with generic value types
/// This provides a bridge between your current RV-based interpreter and the new abstraction
pub struct GenericInterpreterAdapter<V: ValueType> {
    inner: Interpreter,
    _phantom: std::marker::PhantomData<V>,
}

impl<V: ValueType + 'static> GenericInterpreterAdapter<V> {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            inner: interpreter,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Evaluate an expression and return the result as the generic type V
    pub fn eval_generic(&mut self, expr: &Expr) -> Result<V, String> {
        self.inner.eval_as::<V>(expr)
    }
    
    /// Interpret source code and return the result as the generic type V
    pub fn interpret_generic(&mut self, source: &str) -> Result<V, String> {
        self.inner.interpret_as::<V>(source)
    }
    
    /// Get a mutable reference to the underlying interpreter
    pub fn inner_mut(&mut self) -> &mut Interpreter {
        &mut self.inner
    }
    
    /// Get a reference to the underlying interpreter
    pub fn inner(&self) -> &Interpreter {
        &self.inner
    }
    
    /// Convert back to the original interpreter
    pub fn into_inner(self) -> Interpreter {
        self.inner
    }
}

/// Simple wrapper that provides type-safe evaluation with your current interpreter
pub struct TypedInterpreter;

impl TypedInterpreter {
    /// Create an RV-based interpreter (backward compatible)
    pub fn new_rv() -> GenericInterpreterAdapter<RvValue> {
        let interpreter = Interpreter::new(None, true);
        GenericInterpreterAdapter::new(interpreter)
    }
    
    /// Create a BSON-based interpreter
    pub fn new_bson() -> GenericInterpreterAdapter<BsonValue> {
        let interpreter = Interpreter::new(None, true);
        GenericInterpreterAdapter::new(interpreter)
    }
    
    /// Create an interpreter that can be switched between value types
    pub fn new_switchable() -> Interpreter {
        Interpreter::new(None, true)
    }
}

/// Demonstration of how to use the new abstraction with your existing interpreter
pub mod examples {
    use super::*;
    use crate::value::traits::eval_binary;
    use crate::value::traits::Operation;
    
    /// Example: Using your existing interpreter with different value types
    pub fn demonstrate_backward_compatibility() {
        // Your existing code continues to work unchanged
        let mut original_interpreter = Interpreter::new(None, true);
        let original_result = original_interpreter.interpret("10 + 20").unwrap();
        println!("Original result: {}", original_result);
        
        // New: Same interpreter, different output type
        let rv_result: RvValue = original_interpreter.interpret_as("10 + 20").unwrap();
        println!("As RvValue: {}", rv_result);
        
        let bson_result: BsonValue = original_interpreter.interpret_as("10 + 20").unwrap();
        println!("As BsonValue: {}", bson_result);
        
        // All results are equivalent
        assert_eq!(original_result.as_number(), rv_result.as_number());
        assert_eq!(rv_result.as_number(), bson_result.as_number());
    }
    
    /// Example: Using the adapter for type-safe operations
    pub fn demonstrate_typed_interpreter() {
        // Type-safe RV interpreter
        let mut rv_interpreter = TypedInterpreter::new_rv();
        let rv_result = rv_interpreter.interpret_generic("5 * 8").unwrap();
        assert_eq!(rv_result.as_number(), Some(40.0));
        
        // Type-safe BSON interpreter
        let mut bson_interpreter = TypedInterpreter::new_bson();
        let bson_result = bson_interpreter.interpret_generic("5 * 8").unwrap();
        assert_eq!(bson_result.as_number(), Some(40.0));
        
        // Results are equivalent
        assert_eq!(rv_result.as_number(), bson_result.as_number());
    }
    
    /// Example: Generic function that works with any interpreter type
    pub fn generic_calculation<V: ValueType + 'static>(
        interpreter: &mut GenericInterpreterAdapter<V>
    ) -> Result<V, String> {
        // This function works with any value type
        let result = interpreter.interpret_generic("(10 + 5) * 2")?;
        Ok(result)
    }
    
    /// Example: Switching between value types dynamically
    pub fn demonstrate_dynamic_switching() {
        let base_interpreter = TypedInterpreter::new_switchable();
        
        // Use as RV interpreter
        let mut rv_adapter = base_interpreter.with_value_type::<RvValue>();
        let rv_result = rv_adapter.interpret_generic("100 / 4").unwrap();
        
        // Convert back and use as BSON interpreter
        let base_interpreter = rv_adapter.into_inner();
        let mut bson_adapter = base_interpreter.with_value_type::<BsonValue>();
        let bson_result = bson_adapter.interpret_generic("100 / 4").unwrap();
        
        // Results are equivalent
        assert_eq!(rv_result.as_number(), bson_result.as_number());
    }
    
    /// Example: Mixed operations between different value types
    pub fn demonstrate_mixed_operations() {
        let mut rv_interpreter = TypedInterpreter::new_rv();
        let mut bson_interpreter = TypedInterpreter::new_bson();
        
        let rv_result = rv_interpreter.interpret_generic("15 + 10").unwrap();
        let bson_result = bson_interpreter.interpret_generic("5 * 5").unwrap();
        
        // Use generic evaluation to combine results (simplified for demo)
        let combined = rv_result.as_number().unwrap_or(0.0) + bson_result.as_number().unwrap_or(0.0);
        println!("Combined result: {}", combined);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::examples::*;
    
    #[test]
    fn test_interpreter_extension_trait() {
        let mut interpreter = Interpreter::new(None, true);
        
        // Test RvValue conversion
        let rv_result: RvValue = interpreter.interpret_as("42").unwrap();
        assert_eq!(rv_result.as_number(), Some(42.0));
        
        // Test BsonValue conversion
        let bson_result: BsonValue = interpreter.interpret_as("42").unwrap();
        assert_eq!(bson_result.as_number(), Some(42.0));
        
        // Results should be equivalent
        assert_eq!(rv_result.as_number(), bson_result.as_number());
    }
    
    #[test]
    fn test_generic_interpreter_adapter() {
        let interpreter = Interpreter::new(None, true);
        let mut adapter = GenericInterpreterAdapter::<RvValue>::new(interpreter);
        
        let result = adapter.interpret_generic("2 + 3").unwrap();
        assert_eq!(result.as_number(), Some(5.0));
    }
    
    #[test]
    fn test_typed_interpreter() {
        let mut rv_interpreter = TypedInterpreter::new_rv();
        let mut bson_interpreter = TypedInterpreter::new_bson();
        
        let rv_result = rv_interpreter.interpret_generic("10 * 3").unwrap();
        let bson_result = bson_interpreter.interpret_generic("10 * 3").unwrap();
        
        assert_eq!(rv_result.as_number(), Some(30.0));
        assert_eq!(bson_result.as_number(), Some(30.0));
        assert_eq!(rv_result.as_number(), bson_result.as_number());
    }
    
    #[test]
    fn test_generic_calculation() {
        let mut rv_adapter = TypedInterpreter::new_rv();
        let mut bson_adapter = TypedInterpreter::new_bson();
        
        let rv_result = generic_calculation(&mut rv_adapter).unwrap();
        let bson_result = generic_calculation(&mut bson_adapter).unwrap();
        
        assert_eq!(rv_result.as_number(), Some(30.0));
        assert_eq!(bson_result.as_number(), Some(30.0));
        assert_eq!(rv_result.as_number(), bson_result.as_number());
    }
    
    #[test]
    fn test_backward_compatibility() {
        demonstrate_backward_compatibility();
    }
    
    #[test]
    fn test_typed_interpreter_demo() {
        demonstrate_typed_interpreter();
    }
    
    #[test]
    fn test_dynamic_switching() {
        demonstrate_dynamic_switching();
    }
    
    #[test]
    fn test_mixed_operations() {
        demonstrate_mixed_operations();
    }
}