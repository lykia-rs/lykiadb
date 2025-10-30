use super::traits::{ValueType, Operation};
use std::marker::PhantomData;

/// Generic evaluation engine that works with any ValueType implementation
/// This enables zero-cost abstraction - the same evaluation logic works
/// with both RV and BSON implementations without runtime overhead
pub struct GenericEvaluator<V: ValueType> {
    _phantom: PhantomData<V>,
}

impl<V: ValueType> GenericEvaluator<V> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
    
    /// Evaluate a binary operation between two values
    #[inline]
    pub fn eval_binary(&self, left: V, right: V, op: Operation) -> V {
        match op {
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
            Operation::Is => V::boolean(self.strict_equality(&left, &right)),
            Operation::IsNot => V::boolean(!self.strict_equality(&left, &right)),
            Operation::Modulo => self.modulo(left, right),
        }
    }
    
    /// Strict equality check (no type coercion)
    #[inline]
    fn strict_equality(&self, left: &V, right: &V) -> bool {
        // Check if types match first
        let left_type = left.get_type();
        let right_type = right.get_type();
        
        if std::mem::discriminant(&left_type) != std::mem::discriminant(&right_type) {
            return false;
        }
        
        left == right
    }
    
    /// Modulo operation
    #[inline]
    fn modulo(&self, left: V, right: V) -> V {
        if let (Some(a), Some(b)) = (left.as_number(), right.as_number()) {
            if b == 0.0 {
                V::undefined()
            } else {
                V::number(a % b)
            }
        } else {
            V::undefined()
        }
    }
    
    /// Unary operations
    #[inline]
    pub fn eval_unary(&self, value: V, op: UnaryOperation) -> V {
        match op {
            UnaryOperation::Not => value.not(),
            UnaryOperation::Negate => {
                if let Some(n) = value.as_number() {
                    V::number(-n)
                } else {
                    V::undefined()
                }
            }
            UnaryOperation::Plus => {
                if let Some(n) = value.as_number() {
                    V::number(n)
                } else {
                    V::undefined()
                }
            }
        }
    }
    
    /// Type conversion operations
    #[inline]
    pub fn convert_to_boolean(&self, value: &V) -> V {
        V::boolean(value.as_bool())
    }
    
    #[inline]
    pub fn convert_to_number(&self, value: &V) -> V {
        if let Some(n) = value.as_number() {
            V::number(n)
        } else {
            V::undefined()
        }
    }
    
    #[inline]
    pub fn convert_to_string(&self, value: &V) -> V {
        if let Some(s) = value.as_string() {
            V::string(s)
        } else {
            // Fallback to display representation
            V::string(value.to_string())
        }
    }
}

/// Unary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperation {
    Not,      // !value
    Negate,   // -value
    Plus,     // +value
}

/// Generic expression evaluator that works with any ValueType
/// This demonstrates how complex evaluation logic can be written once
/// and work with any implementation of ValueType
pub struct ExpressionEvaluator<V: ValueType> {
    evaluator: GenericEvaluator<V>,
}

impl<V: ValueType> ExpressionEvaluator<V> {
    pub fn new() -> Self {
        Self {
            evaluator: GenericEvaluator::new(),
        }
    }
    
    /// Evaluate a complex expression tree
    pub fn evaluate_expression(&self, expr: &Expression<V>) -> V {
        match expr {
            Expression::Literal(value) => value.clone(),
            Expression::Binary { left, op, right } => {
                let left_val = self.evaluate_expression(left);
                let right_val = self.evaluate_expression(right);
                self.evaluator.eval_binary(left_val, right_val, *op)
            }
            Expression::Unary { op, operand } => {
                let operand_val = self.evaluate_expression(operand);
                self.evaluator.eval_unary(operand_val, *op)
            }
            Expression::Conditional { condition, if_true, if_false } => {
                let condition_val = self.evaluate_expression(condition);
                if condition_val.as_bool() {
                    self.evaluate_expression(if_true)
                } else {
                    self.evaluate_expression(if_false)
                }
            }
        }
    }
}

/// Simple expression AST for demonstration
#[derive(Debug, Clone)]
pub enum Expression<V: ValueType> {
    Literal(V),
    Binary {
        left: Box<Expression<V>>,
        op: Operation,
        right: Box<Expression<V>>,
    },
    Unary {
        op: UnaryOperation,
        operand: Box<Expression<V>>,
    },
    Conditional {
        condition: Box<Expression<V>>,
        if_true: Box<Expression<V>>,
        if_false: Box<Expression<V>>,
    },
}

impl<V: ValueType> Expression<V> {
    /// Create a literal expression
    pub fn literal(value: V) -> Self {
        Expression::Literal(value)
    }
    
    /// Create a binary expression
    pub fn binary(left: Expression<V>, op: Operation, right: Expression<V>) -> Self {
        Expression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }
    
    /// Create a unary expression
    pub fn unary(op: UnaryOperation, operand: Expression<V>) -> Self {
        Expression::Unary {
            op,
            operand: Box::new(operand),
        }
    }
    
    /// Create a conditional expression
    pub fn conditional(condition: Expression<V>, if_true: Expression<V>, if_false: Expression<V>) -> Self {
        Expression::Conditional {
            condition: Box::new(condition),
            if_true: Box::new(if_true),
            if_false: Box::new(if_false),
        }
    }
}

/// Collection of utility functions that work with any ValueType
pub struct ValueUtils;

impl ValueUtils {
    /// Check if two values are equivalent (with type coercion)
    pub fn are_equivalent<V: ValueType>(a: &V, b: &V) -> bool {
        // Try direct equality first
        if a == b {
            return true;
        }
        
        // Try numeric coercion
        if let (Some(num_a), Some(num_b)) = (a.as_number(), b.as_number()) {
            return num_a == num_b;
        }
        
        // Try boolean coercion
        if a.is_boolean() || b.is_boolean() {
            return a.as_bool() == b.as_bool();
        }
        
        false
    }
    
    /// Deep clone a value (useful for immutable operations)
    pub fn deep_clone<V: ValueType>(value: &V) -> V {
        // Since ValueType requires Clone, this is simple
        value.clone()
    }
    
    /// Get a human-readable type name
    pub fn type_name<V: ValueType>(value: &V) -> &'static str {
        use lykiadb_lang::types::Datatype;
        match value.get_type() {
            Datatype::Str => "string",
            Datatype::Num => "number",
            Datatype::Bool => "boolean",
            Datatype::Array(_) => "array",
            Datatype::Object(_) => "object",
            Datatype::Callable(_, _) => "function",
            Datatype::Datatype => "type",
            Datatype::None => "undefined",
            _ => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::rv_wrapper::RvValue;
    use crate::value::bson_wrapper::BsonValue;
    
    /// Test that the same expression evaluates identically with both implementations
    #[test]
    fn test_zero_cost_abstraction() {
        // Create identical expressions for both value types
        let rv_expr = Expression::binary(
            Expression::literal(RvValue::number(42.0)),
            Operation::Add,
            Expression::literal(RvValue::number(24.0))
        );
        
        let bson_expr = Expression::binary(
            Expression::literal(BsonValue::number(42.0)),
            Operation::Add,
            Expression::literal(BsonValue::number(24.0))
        );
        
        // Evaluate with the same generic evaluator
        let rv_evaluator = ExpressionEvaluator::new();
        let bson_evaluator = ExpressionEvaluator::new();
        
        let rv_result = rv_evaluator.evaluate_expression(&rv_expr);
        let bson_result = bson_evaluator.evaluate_expression(&bson_expr);
        
        // Results should be equivalent
        assert_eq!(rv_result.as_number(), Some(66.0));
        assert_eq!(bson_result.as_number(), Some(66.0));
        assert_eq!(rv_result.to_string(), bson_result.to_string());
    }
    
    /// Test that generic functions compile and work with both types
    fn generic_test<V: ValueType>() {
        let evaluator = GenericEvaluator::new();
        
        let a = V::number(10.0);
        let b = V::number(5.0);
        
        // Test all operations
        let sum = evaluator.eval_binary(a.clone(), b.clone(), Operation::Add);
        assert_eq!(sum.as_number(), Some(15.0));
        
        let diff = evaluator.eval_binary(a.clone(), b.clone(), Operation::Subtract);
        assert_eq!(diff.as_number(), Some(5.0));
        
        let lt = evaluator.eval_binary(b.clone(), a.clone(), Operation::Less);
        assert_eq!(lt.as_bool(), true);
        
        // Test unary operations
        let neg = evaluator.eval_unary(a.clone(), UnaryOperation::Negate);
        assert_eq!(neg.as_number(), Some(-10.0));
        
        let not = evaluator.eval_unary(V::boolean(false), UnaryOperation::Not);
        assert_eq!(not.as_bool(), true);
    }
    
    #[test]
    fn test_rv_generic_operations() {
        generic_test::<RvValue>();
    }
    
    #[test]
    fn test_bson_generic_operations() {
        generic_test::<BsonValue>();
    }
    
    #[test]
    fn test_conditional_expressions() {
        let rv_expr = Expression::conditional(
            Expression::literal(RvValue::boolean(true)),
            Expression::literal(RvValue::string("yes".to_string())),
            Expression::literal(RvValue::string("no".to_string()))
        );
        
        let evaluator = ExpressionEvaluator::new();
        let result = evaluator.evaluate_expression(&rv_expr);
        
        assert_eq!(result.as_string(), Some("yes".to_string()));
    }
}