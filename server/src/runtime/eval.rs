use crate::lang::ast::expr::Operation;
use crate::runtime::types::RV;

#[inline(always)]
pub fn eval_binary(left_eval: RV, right_eval: RV, operation: Operation) -> RV {
    /*
        TODO(vck):
            - Add support for object operations
            - Add support for array operations
            - Add support for function operations
    */
    match operation {
        Operation::IsEqual => RV::Bool(left_eval == right_eval),
        Operation::IsNotEqual => RV::Bool(left_eval != right_eval),
        Operation::Less => RV::Bool(left_eval < right_eval),
        Operation::LessEqual => RV::Bool(left_eval <= right_eval),
        Operation::Greater => RV::Bool(left_eval > right_eval),
        Operation::GreaterEqual => RV::Bool(left_eval >= right_eval),
        Operation::Add => left_eval + right_eval,
        Operation::Subtract => left_eval - right_eval,
        Operation::Multiply => left_eval * right_eval,
        Operation::Divide => left_eval / right_eval,
        _ => RV::Undefined,
    }
}

#[cfg(test)]
mod test {
    use std::{f64::INFINITY, sync::Arc};

    use crate::{
        lang::ast::expr::Operation,
        runtime::{eval::eval_binary, types::RV},
    };

    #[test]
    fn test_eval_binary_addition() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Add),
            RV::Num(3.0)
        );
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::Add),
            RV::Num(3.0)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::Add
            ),
            RV::Str(Arc::new("ab".to_string()))
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Add
            ),
            RV::Str(Arc::new("ba".to_string()))
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::Add),
            RV::Num(2.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::Add),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Add),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Add),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Add),
            RV::Num(2.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Add),
            RV::Num(2.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::Add),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::Add),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Add),
            RV::Num(2.0)
        );
    }

    #[test]
    fn test_eval_binary_subtraction() {
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::Subtract),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Subtract),
            RV::Num(-1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::Subtract),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::Subtract),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Subtract),
            RV::Num(-1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Subtract),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Subtract),
            RV::Num(0.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Subtract),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::Subtract),
            RV::Num(-1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::Subtract),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Subtract),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::Subtract
            ),
            RV::NaN
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Subtract
            ),
            RV::NaN
        );
    }

    #[test]
    fn test_eval_binary_multiplication() {
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::Multiply),
            RV::Num(2.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Multiply),
            RV::Num(2.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::Multiply),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::Multiply),
            RV::Num(0.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Multiply),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Multiply),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Multiply),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Multiply),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::Multiply),
            RV::Num(0.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::Multiply),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Multiply),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::Multiply
            ),
            RV::NaN
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Multiply
            ),
            RV::NaN
        );
    }

    #[test]
    fn test_eval_binary_division() {
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::Divide),
            RV::Num(2.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Divide),
            RV::Num(0.5)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::Divide),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::Divide),
            RV::Num(INFINITY)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Divide),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Divide),
            RV::NaN
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Divide),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Divide),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::Divide),
            RV::Num(0.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::Divide),
            RV::Num(INFINITY)
        );
        //
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Divide),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::Divide
            ),
            RV::NaN
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Divide
            ),
            RV::NaN
        );
    }

    #[test]
    fn test_eval_binary_is_equal() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(1.0), Operation::IsEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::IsEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::IsEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::IsEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::IsEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::IsEqual),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Num(1.0),
                RV::Str(Arc::new("a".to_string())),
                Operation::IsEqual
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Num(1.0),
                Operation::IsEqual
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::IsEqual
            ),
            RV::Bool(false)
        );
    }

    #[test]
    fn test_eval_binary_is_not_equal() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(1.0), Operation::IsNotEqual),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::IsNotEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::IsNotEqual),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::IsNotEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::IsNotEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::IsNotEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Num(1.0),
                RV::Str(Arc::new("a".to_string())),
                Operation::IsNotEqual
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Num(1.0),
                Operation::IsNotEqual
            ),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::IsNotEqual
            ),
            RV::Bool(true)
        );
    }

    #[test]
    fn test_eval_binary_less() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(1.0), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Less),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::Less),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Less),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Less),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::Less),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::Less),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Num(1.0),
                RV::Str(Arc::new("a".to_string())),
                Operation::Less
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Num(1.0),
                Operation::Less
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Less
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::Less
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Less
            ),
            RV::Bool(false)
        );
    }

    #[test]
    fn test_eval_binary_less_equal() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(1.0), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::LessEqual),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::LessEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::LessEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::LessEqual),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Num(1.0),
                RV::Str(Arc::new("a".to_string())),
                Operation::LessEqual
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Num(1.0),
                Operation::LessEqual
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::LessEqual
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::LessEqual
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::LessEqual
            ),
            RV::Bool(false)
        );
    }

    #[test]
    fn test_eval_binary_greater() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(1.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::Greater),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::Greater),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Greater),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::Greater),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Num(1.0),
                RV::Str(Arc::new("a".to_string())),
                Operation::Greater
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Num(1.0),
                Operation::Greater
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Greater
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::Greater
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Greater
            ),
            RV::Bool(true)
        );
    }

    #[test]
    fn test_eval_binary_greater_equal() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::GreaterEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::GreaterEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::GreaterEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::GreaterEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Num(1.0),
                RV::Str(Arc::new("a".to_string())),
                Operation::GreaterEqual
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Num(1.0),
                Operation::GreaterEqual
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::GreaterEqual
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::GreaterEqual
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::GreaterEqual
            ),
            RV::Bool(true)
        );
    }

    #[test]
    fn test_eval_binary_nan() {
        assert_eq!(eval_binary(RV::NaN, RV::Num(1.0), Operation::Add), RV::NaN);
        assert_eq!(eval_binary(RV::Num(1.0), RV::NaN, Operation::Add), RV::NaN);
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::Subtract),
            RV::NaN
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::Subtract),
            RV::NaN
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::Multiply),
            RV::NaN
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::Multiply),
            RV::NaN
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::Divide),
            RV::NaN
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::Divide),
            RV::NaN
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::LessEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::LessEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::GreaterEqual),
            RV::Bool(false)
        );
    }

    #[test]
    fn test_eval_binary_coercion() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Add),
            RV::Num(2.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Subtract),
            RV::Num(0.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Multiply),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Divide),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::IsEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::IsNotEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Bool(true),
                Operation::Add
            ),
            RV::Str(Arc::new("atrue".to_string()))
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Bool(true),
                Operation::Less
            ),
            RV::Bool(false)
        );
        //

        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Add),
            RV::Num(2.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Subtract),
            RV::Num(0.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Multiply),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Divide),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::IsEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::IsNotEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Bool(true),
                RV::Str(Arc::new("a".to_string())),
                Operation::Add
            ),
            RV::Str(Arc::new("truea".to_string()))
        );
        assert_eq!(
            eval_binary(
                RV::Bool(true),
                RV::Str(Arc::new("a".to_string())),
                Operation::Less
            ),
            RV::Bool(false)
        );
    }
}
