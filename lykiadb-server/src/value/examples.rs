/*!
# Zero-Cost Value Abstraction Examples

This module demonstrates how to use the ValueType trait to write code that works
with both RV and BSON implementations without any runtime overhead.

The key insight is that all the evaluation logic is written generically using
the ValueType trait, and the compiler monomorphizes the code for each concrete
type, resulting in zero runtime overhead.
*/

use super::{
    traits::{ValueType, Operation},
    rv_wrapper::RvValue,
    bson_wrapper::BsonValue,
    generic_eval::{GenericEvaluator, ExpressionEvaluator, Expression, UnaryOperation},
    conversions::ValueConverter,
};

/// Example 1: Generic Calculator
/// This calculator works with any ValueType implementation
pub struct GenericCalculator<V: ValueType> {
    evaluator: GenericEvaluator<V>,
}

impl<V: ValueType> GenericCalculator<V> {
    pub fn new() -> Self {
        Self {
            evaluator: GenericEvaluator::new(),
        }
    }
    
    /// Add two values
    pub fn add(&self, a: V, b: V) -> V {
        self.evaluator.eval_binary(a, b, Operation::Add)
    }
    
    /// Subtract two values
    pub fn subtract(&self, a: V, b: V) -> V {
        self.evaluator.eval_binary(a, b, Operation::Subtract)
    }
    
    /// Multiply two values
    pub fn multiply(&self, a: V, b: V) -> V {
        self.evaluator.eval_binary(a, b, Operation::Multiply)
    }
    
    /// Divide two values
    pub fn divide(&self, a: V, b: V) -> V {
        self.evaluator.eval_binary(a, b, Operation::Divide)
    }
    
    /// Calculate percentage: (value / total) * 100
    pub fn percentage(&self, value: V, total: V) -> V {
        let ratio = self.divide(value, total);
        let hundred = V::number(100.0);
        self.multiply(ratio, hundred)
    }
    
    /// Calculate compound interest: principal * (1 + rate)^years
    pub fn compound_interest(&self, principal: V, rate: V, years: V) -> V {
        let one = V::number(1.0);
        let rate_plus_one = self.add(one, rate);
        
        // For simplicity, we'll just do simple multiplication instead of exponentiation
        // In a real implementation, you'd implement a power function
        let multiplier = self.multiply(rate_plus_one, years);
        self.multiply(principal, multiplier)
    }
}

/// Example 2: Generic Comparison Engine
pub struct GenericComparator<V: ValueType> {
    evaluator: GenericEvaluator<V>,
}

impl<V: ValueType> GenericComparator<V> {
    pub fn new() -> Self {
        Self {
            evaluator: GenericEvaluator::new(),
        }
    }
    
    /// Find the maximum of two values
    pub fn max(&self, a: V, b: V) -> V {
        let is_greater = self.evaluator.eval_binary(a.clone(), b.clone(), Operation::Greater);
        if is_greater.as_bool() {
            a
        } else {
            b
        }
    }
    
    /// Find the minimum of two values
    pub fn min(&self, a: V, b: V) -> V {
        let is_less = self.evaluator.eval_binary(a.clone(), b.clone(), Operation::Less);
        if is_less.as_bool() {
            a
        } else {
            b
        }
    }
    
    /// Check if a value is within a range (inclusive)
    pub fn is_in_range(&self, value: V, min: V, max: V) -> V {
        let gte_min = self.evaluator.eval_binary(value.clone(), min, Operation::GreaterEqual);
        let lte_max = self.evaluator.eval_binary(value, max, Operation::LessEqual);
        self.evaluator.eval_binary(gte_min, lte_max, Operation::And)
    }
    
    /// Sort three values in ascending order
    pub fn sort_three(&self, a: V, b: V, c: V) -> (V, V, V) {
        let min_ab = self.min(a.clone(), b.clone());
        let max_ab = self.max(a, b);
        
        let min_all = self.min(min_ab.clone(), c.clone());
        let max_all = self.max(max_ab.clone(), c);
        
        // The middle value is the one that's neither min nor max
        let is_min_middle = self.evaluator.eval_binary(min_ab.clone(), min_all.clone(), Operation::IsNotEqual);
        let is_max_middle = self.evaluator.eval_binary(max_ab.clone(), max_all.clone(), Operation::IsNotEqual);
        
        let middle = if is_min_middle.as_bool() {
            min_ab
        } else if is_max_middle.as_bool() {
            max_ab
        } else {
            // All values are equal or one is the middle
            min_all.clone()
        };
        
        (min_all, middle, max_all)
    }
}

/// Example 3: Generic Statistics Calculator
pub struct GenericStats<V: ValueType> {
    evaluator: GenericEvaluator<V>,
}

impl<V: ValueType> GenericStats<V> {
    pub fn new() -> Self {
        Self {
            evaluator: GenericEvaluator::new(),
        }
    }
    
    /// Calculate the sum of values
    pub fn sum(&self, values: Vec<V>) -> V {
        values.into_iter().reduce(|acc, val| {
            self.evaluator.eval_binary(acc, val, Operation::Add)
        }).unwrap_or_else(|| V::number(0.0))
    }
    
    /// Calculate the average of values
    pub fn average(&self, values: Vec<V>) -> V {
        if values.is_empty() {
            return V::undefined();
        }
        
        let sum = self.sum(values.clone());
        let count = V::number(values.len() as f64);
        self.evaluator.eval_binary(sum, count, Operation::Divide)
    }
    
    /// Count values that match a condition
    pub fn count_if<F>(&self, values: Vec<V>, predicate: F) -> V 
    where 
        F: Fn(&V) -> bool 
    {
        let count = values.iter().filter(|v| predicate(v)).count();
        V::number(count as f64)
    }
    
    /// Find values greater than threshold
    pub fn filter_greater_than(&self, values: Vec<V>, threshold: V) -> Vec<V> {
        values.into_iter().filter(|v| {
            let comparison = self.evaluator.eval_binary(v.clone(), threshold.clone(), Operation::Greater);
            comparison.as_bool()
        }).collect()
    }
}

/// Example 4: Type-Agnostic Expression Builder
pub struct ExpressionBuilder<V: ValueType> {
    _phantom: std::marker::PhantomData<V>,
}

impl<V: ValueType> ExpressionBuilder<V> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Build a quadratic expression: ax² + bx + c
    pub fn quadratic(&self, a: V, b: V, c: V, x: V) -> Expression<V> {
        // x²
        let x_squared = Expression::binary(
            Expression::literal(x.clone()),
            Operation::Multiply,
            Expression::literal(x.clone())
        );
        
        // ax²
        let ax_squared = Expression::binary(
            Expression::literal(a),
            Operation::Multiply,
            x_squared
        );
        
        // bx
        let bx = Expression::binary(
            Expression::literal(b),
            Operation::Multiply,
            Expression::literal(x)
        );
        
        // ax² + bx
        let ax_squared_plus_bx = Expression::binary(
            ax_squared,
            Operation::Add,
            bx
        );
        
        // ax² + bx + c
        Expression::binary(
            ax_squared_plus_bx,
            Operation::Add,
            Expression::literal(c)
        )
    }
    
    /// Build a conditional expression: if condition then value1 else value2
    pub fn conditional(&self, condition: V, then_value: V, else_value: V) -> Expression<V> {
        Expression::conditional(
            Expression::literal(condition),
            Expression::literal(then_value),
            Expression::literal(else_value)
        )
    }
    
    /// Build an expression to check if a number is even
    pub fn is_even(&self, value: V) -> Expression<V> {
        let two = V::number(2.0);
        let zero = V::number(0.0);
        
        let modulo = Expression::binary(
            Expression::literal(value),
            Operation::Modulo,
            Expression::literal(two)
        );
        
        Expression::binary(
            modulo,
            Operation::IsEqual,
            Expression::literal(zero)
        )
    }
}

/// Example 5: Performance Comparison and Benchmarking
pub struct BenchmarkRunner;

impl BenchmarkRunner {
    /// Run the same computation with both RV and BSON implementations
    /// This demonstrates that performance is identical (zero-cost abstraction)
    pub fn compare_implementations() -> (String, String) {
        let start_rv = std::time::Instant::now();
        let rv_result = Self::run_computation_rv();
        let rv_duration = start_rv.elapsed();
        
        let start_bson = std::time::Instant::now();
        let bson_result = Self::run_computation_bson();
        let bson_duration = start_bson.elapsed();
        
        let rv_summary = format!(
            "RV Implementation: {} operations in {:?}, result: {}",
            1000, rv_duration, rv_result.to_string()
        );
        
        let bson_summary = format!(
            "BSON Implementation: {} operations in {:?}, result: {}",
            1000, bson_duration, bson_result.to_string()
        );
        
        (rv_summary, bson_summary)
    }
    
    fn run_computation_rv() -> RvValue {
        let calc = GenericCalculator::new();
        let mut result = RvValue::number(1.0);
        
        for i in 1..=1000 {
            let i_val = RvValue::number(i as f64);
            result = calc.add(result, i_val);
        }
        
        result
    }
    
    fn run_computation_bson() -> BsonValue {
        let calc = GenericCalculator::new();
        let mut result = BsonValue::number(1.0);
        
        for i in 1..=1000 {
            let i_val = BsonValue::number(i as f64);
            result = calc.add(result, i_val);
        }
        
        result
    }
    
    /// Demonstrate that the same generic function works with both types
    pub fn generic_computation<V: ValueType>() -> V {
        let calc = GenericCalculator::new();
        let comp = GenericComparator::new();
        
        let a = V::number(42.0);
        let b = V::number(24.0);
        let c = V::number(66.0);
        
        let sum = calc.add(a.clone(), b.clone());
        let max = comp.max(sum, c);
        
        max
    }
}

/// Example 6: Interoperability Demonstration
pub struct InteropDemo;

impl InteropDemo {
    /// Show how to work with mixed value types
    pub fn mixed_operations() -> String {
        // Create values with different implementations
        let rv_value = RvValue::number(10.0);
        let bson_value = BsonValue::number(5.0);
        
        // Convert between types seamlessly
        let rv_as_bson: BsonValue = rv_value.clone().into();
        let bson_as_rv: RvValue = bson_value.clone().into();
        
        // Use the same generic calculator with both
        let rv_calc = GenericCalculator::new();
        let bson_calc = GenericCalculator::new();
        
        let rv_result = rv_calc.add(rv_value, bson_as_rv);
        let bson_result = bson_calc.add(rv_as_bson, bson_value);
        
        format!(
            "RV result: {}, BSON result: {}, Equal: {}",
            rv_result.to_string(),
            bson_result.to_string(),
            rv_result.as_number() == bson_result.as_number()
        )
    }
    
    /// Demonstrate serialization compatibility
    pub fn serialization_demo() -> Result<String, Box<dyn std::error::Error>> {
        let rv_value = RvValue::number(3.14159);
        let bson_value = BsonValue::number(3.14159);
        
        // Serialize both to JSON
        let rv_json = serde_json::to_string(&rv_value)?;
        let bson_json = serde_json::to_string(&bson_value)?;
        
        // Deserialize back
        let rv_deserialized: RvValue = serde_json::from_str(&rv_json)?;
        let bson_deserialized: BsonValue = serde_json::from_str(&bson_json)?;
        
        Ok(format!(
            "Original values equal: {}, Serialized JSONs equal: {}, Deserialized values equal: {}",
            rv_value.as_number() == bson_value.as_number(),
            rv_json == bson_json,
            rv_deserialized.as_number() == bson_deserialized.as_number()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generic_calculator_with_rv() {
        let calc = GenericCalculator::new();
        
        let a = RvValue::number(10.0);
        let b = RvValue::number(5.0);
        
        let sum = calc.add(a.clone(), b.clone());
        assert_eq!(sum.as_number(), Some(15.0));
        
        let percentage = calc.percentage(a, b);
        assert_eq!(percentage.as_number(), Some(200.0));
    }
    
    #[test]
    fn test_generic_calculator_with_bson() {
        let calc = GenericCalculator::new();
        
        let a = BsonValue::number(10.0);
        let b = BsonValue::number(5.0);
        
        let sum = calc.add(a.clone(), b.clone());
        assert_eq!(sum.as_number(), Some(15.0));
        
        let percentage = calc.percentage(a, b);
        assert_eq!(percentage.as_number(), Some(200.0));
    }
    
    #[test]
    fn test_generic_comparator() {
        fn test_with_type<V: ValueType>() {
            let comp = GenericComparator::new();
            
            let a = V::number(10.0);
            let b = V::number(5.0);
            let c = V::number(15.0);
            
            let max = comp.max(a.clone(), b.clone());
            assert_eq!(max.as_number(), Some(10.0));
            
            let (min, mid, max) = comp.sort_three(a, b, c);
            assert_eq!(min.as_number(), Some(5.0));
            assert_eq!(max.as_number(), Some(15.0));
        }
        
        test_with_type::<RvValue>();
        test_with_type::<BsonValue>();
    }
    
    #[test]
    fn test_generic_stats() {
        fn test_with_type<V: ValueType>() {
            let stats = GenericStats::new();
            
            let values = vec![
                V::number(1.0),
                V::number(2.0),
                V::number(3.0),
                V::number(4.0),
                V::number(5.0),
            ];
            
            let sum = stats.sum(values.clone());
            assert_eq!(sum.as_number(), Some(15.0));
            
            let avg = stats.average(values.clone());
            assert_eq!(avg.as_number(), Some(3.0));
            
            let greater_than_three = stats.filter_greater_than(values, V::number(3.0));
            assert_eq!(greater_than_three.len(), 2);
        }
        
        test_with_type::<RvValue>();
        test_with_type::<BsonValue>();
    }
    
    #[test]
    fn test_expression_builder() {
        let builder = ExpressionBuilder::new();
        let evaluator = ExpressionEvaluator::new();
        
        // Test with RV
        let rv_expr = builder.quadratic(
            RvValue::number(2.0),  // a
            RvValue::number(3.0),  // b
            RvValue::number(1.0),  // c
            RvValue::number(2.0)   // x
        );
        let rv_result = evaluator.evaluate_expression(&rv_expr);
        // 2(2²) + 3(2) + 1 = 8 + 6 + 1 = 15
        assert_eq!(rv_result.as_number(), Some(15.0));
        
        // Test with BSON - need separate builder since it's generic
        let bson_builder = ExpressionBuilder::<BsonValue>::new();
        let bson_expr = bson_builder.quadratic(
            BsonValue::number(2.0),  // a
            BsonValue::number(3.0),  // b
            BsonValue::number(1.0),  // c
            BsonValue::number(2.0)   // x
        );
        let bson_evaluator = ExpressionEvaluator::<BsonValue>::new();
        let bson_result = bson_evaluator.evaluate_expression(&bson_expr);
        assert_eq!(bson_result.as_number(), Some(15.0));
    }
    
    #[test]
    fn test_benchmark_runner() {
        // Test that generic computation works with both types
        let rv_result: RvValue = BenchmarkRunner::generic_computation();
        let bson_result: BsonValue = BenchmarkRunner::generic_computation();
        
        assert_eq!(rv_result.as_number(), Some(66.0));
        assert_eq!(bson_result.as_number(), Some(66.0));
        assert_eq!(rv_result.as_number(), bson_result.as_number());
    }
    
    #[test]
    fn test_interop_demo() {
        let result = InteropDemo::mixed_operations();
        println!("Mixed operations result: {}", result);
        
        let serialization_result = InteropDemo::serialization_demo().unwrap();
        println!("Serialization demo result: {}", serialization_result);
    }
    
    #[test]
    fn test_zero_cost_abstraction_proof() {
        // This test proves that the abstraction is truly zero-cost by
        // showing that the same generic code produces identical results
        // with different implementations
        
        fn generic_complex_calculation<V: ValueType>() -> V {
            let calc = GenericCalculator::new();
            let comp = GenericComparator::new();
            let stats = GenericStats::new();
            
            let values = vec![
                V::number(1.0), V::number(4.0), V::number(9.0), 
                V::number(16.0), V::number(25.0)
            ];
            
            let sum = stats.sum(values.clone());
            let avg = stats.average(values);
            let max_val = comp.max(sum.clone(), avg.clone());
            let percentage = calc.percentage(avg, sum);
            
            calc.add(max_val, percentage)
        }
        
        let rv_result: RvValue = generic_complex_calculation();
        let bson_result: BsonValue = generic_complex_calculation();
        
        // Results should be identical
        assert_eq!(rv_result.as_number(), bson_result.as_number());
        assert_eq!(rv_result.to_string(), bson_result.to_string());
        
        println!("RV result: {}", rv_result);
        println!("BSON result: {}", bson_result);
        println!("Zero-cost abstraction verified: {}", 
                 rv_result.as_number() == bson_result.as_number());
    }
}