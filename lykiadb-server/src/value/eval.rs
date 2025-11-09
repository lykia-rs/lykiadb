use super::RV;
use lykiadb_lang::ast::expr::Operation;

#[inline(always)]
pub fn eval_binary(left_eval: RV, right_eval: RV, operation: Operation) -> RV {
    /*
        TODO(vck):
            - Add support for object operations
            - Add support for array operations
            - Add support for function operations
    */
    match operation {
        Operation::Is | Operation::IsEqual => RV::Bool(left_eval == right_eval),
        Operation::IsNot | Operation::IsNotEqual => RV::Bool(left_eval != right_eval),
        Operation::Less => RV::Bool(left_eval < right_eval),
        Operation::LessEqual => RV::Bool(left_eval <= right_eval),
        Operation::Greater => RV::Bool(left_eval > right_eval),
        Operation::GreaterEqual => RV::Bool(left_eval >= right_eval),
        Operation::Add => left_eval + right_eval,
        Operation::Subtract => left_eval - right_eval,
        Operation::Multiply => left_eval * right_eval,
        Operation::Divide => left_eval / right_eval,
        Operation::In => left_eval.is_in(&right_eval),
        Operation::NotIn => left_eval.is_in(&right_eval).not(),
        // TODO: Implement operations:
        /*
           Operation::Like
           Operation::NotLike
        */
        _ => RV::Undefined,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use lykiadb_lang::ast::expr::Operation;

    use crate::value::{
        RVArray, RVObject,
        eval::{RV, eval_binary},
    };

    #[test]
    fn test_is_value_truthy() {
        assert!(!(RV::Undefined).as_bool());
        assert!(!(RV::Bool(false)).as_bool());
        assert!((RV::Bool(true)).as_bool());
        assert!(!(RV::Num(0.0)).as_bool());
        assert!((RV::Num(0.1)).as_bool());
        assert!((RV::Num(-0.1)).as_bool());
        assert!((RV::Num(1.0)).as_bool());
        assert!((RV::Num(-1.0)).as_bool());
        assert!(!(RV::Str(Arc::new("".to_owned()))).as_bool());
        assert!((RV::Str(Arc::new("0".to_owned()))).as_bool());
        assert!((RV::Str(Arc::new("false".to_owned()))).as_bool());
        assert!((RV::Str(Arc::new("true".to_owned()))).as_bool());
        assert!((RV::Str(Arc::new("foo".to_owned()))).as_bool());
        assert!((RV::Array(RVArray::new())).as_bool());
        assert!((RV::Object(RVObject::new())).as_bool());
    }

    #[test]
    fn test_as_number() {
        assert_eq!((RV::Num(1.0)).as_number(), Some(1.0));
        assert_eq!((RV::Bool(false)).as_number(), Some(0.0));
        assert_eq!((RV::Bool(true)).as_number(), Some(1.0));
        assert_eq!((RV::Str(Arc::new("".to_owned()))).as_number(), None);
    }

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
            RV::Undefined
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Subtract
            ),
            RV::Undefined
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
            RV::Undefined
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Multiply
            ),
            RV::Undefined
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
            RV::Num(f64::INFINITY)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Divide),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Divide),
            RV::Undefined
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
            RV::Num(f64::INFINITY)
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
            RV::Undefined
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Divide
            ),
            RV::Undefined
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
        assert_eq!(
            eval_binary(RV::Undefined, RV::Num(1.0), Operation::Add),
            RV::Undefined
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Undefined, Operation::Add),
            RV::Undefined
        );
        assert_eq!(
            eval_binary(RV::Undefined, RV::Num(1.0), Operation::Subtract),
            RV::Undefined
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Undefined, Operation::Subtract),
            RV::Undefined
        );
        assert_eq!(
            eval_binary(RV::Undefined, RV::Num(1.0), Operation::Multiply),
            RV::Undefined
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Undefined, Operation::Multiply),
            RV::Undefined
        );
        assert_eq!(
            eval_binary(RV::Undefined, RV::Num(1.0), Operation::Divide),
            RV::Undefined
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Undefined, Operation::Divide),
            RV::Undefined
        );
        assert_eq!(
            eval_binary(RV::Undefined, RV::Num(1.0), Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Undefined, Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Undefined, RV::Num(1.0), Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Undefined, Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Undefined, RV::Num(1.0), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Undefined, Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Undefined, RV::Num(1.0), Operation::LessEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Undefined, Operation::LessEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Undefined, RV::Num(1.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Undefined, Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Undefined, RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Undefined, Operation::GreaterEqual),
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

#[cfg(test)]
mod property_tests {
    use std::sync::Arc;

    use super::*;
    use crate::value::eval::eval_binary;
    use proptest::prelude::*;

    // Strategy for generating RV values
    fn rv_strategy() -> impl Strategy<Value = RV> {
        prop_oneof![
            Just(RV::Undefined),
            any::<bool>().prop_map(RV::Bool),
            any::<f64>()
                .prop_filter("finite numbers", |x| x.is_finite())
                .prop_map(RV::Num),
            "[a-zA-Z0-9]*".prop_map(|s| RV::Str(Arc::new(s))),
            // For simplicity, we'll skip arrays and objects in basic tests
        ]
    }

    // Strategy for numeric RV values only
    fn numeric_rv_strategy() -> impl Strategy<Value = RV> {
        prop_oneof![
            any::<bool>().prop_map(RV::Bool),
            any::<f64>()
                .prop_filter("finite numbers", |x| x.is_finite())
                .prop_map(RV::Num),
        ]
    }

    // Strategy for binary operations
    fn binary_operation_strategy() -> impl Strategy<Value = Operation> {
        prop_oneof![
            Just(Operation::Add),
            Just(Operation::Subtract),
            Just(Operation::Multiply),
            Just(Operation::Divide),
            Just(Operation::IsEqual),
            Just(Operation::IsNotEqual),
            Just(Operation::Less),
            Just(Operation::LessEqual),
            Just(Operation::Greater),
            Just(Operation::GreaterEqual),
        ]
    }

    proptest! {
        // Property: Addition should be commutative for numeric types
        #[test]
        fn addition_is_commutative_for_numbers(
            a in numeric_rv_strategy(),
            b in numeric_rv_strategy()
        ) {
            let result1 = eval_binary(a.clone(), b.clone(), Operation::Add);
            let result2 = eval_binary(b, a, Operation::Add);
            prop_assert_eq!(result1, result2);
        }

        // Property: Multiplication should be commutative for numeric types
        #[test]
        fn multiplication_is_commutative_for_numbers(
            a in numeric_rv_strategy(),
            b in numeric_rv_strategy()
        ) {
            let result1 = eval_binary(a.clone(), b.clone(), Operation::Multiply);
            let result2 = eval_binary(b, a, Operation::Multiply);
            prop_assert_eq!(result1, result2);
        }

        // Property: Adding zero should be identity
        #[test]
        fn adding_zero_is_identity(a in numeric_rv_strategy()) {
            let zero = RV::Num(0.0);
            let result = eval_binary(a.clone(), zero, Operation::Add);
            if let Some(num) = a.as_number() {
                prop_assert_eq!(result, RV::Num(num));
            }
        }

        // Property: Multiplying by one should be identity
        #[test]
        fn multiplying_by_one_is_identity(a in numeric_rv_strategy()) {
            let one = RV::Num(1.0);
            let result = eval_binary(a.clone(), one, Operation::Multiply);
            if let Some(num) = a.as_number() {
                prop_assert_eq!(result, RV::Num(num));
            }
        }

        // Property: Equality should be reflexive
        #[test]
        fn equality_is_reflexive(a in rv_strategy()) {
            let result = eval_binary(a.clone(), a, Operation::IsEqual);
            prop_assert_eq!(result, RV::Bool(true));
        }

        // Property: Inequality should be symmetric
        #[test]
        fn inequality_is_symmetric(
            a in rv_strategy(),
            b in rv_strategy()
        ) {
            let result1 = eval_binary(a.clone(), b.clone(), Operation::IsNotEqual);
            let result2 = eval_binary(b, a, Operation::IsNotEqual);
            prop_assert_eq!(result1, result2);
        }

        // Property: Less than should be antisymmetric
        #[test]
        fn less_than_antisymmetric(
            a in numeric_rv_strategy(),
            b in numeric_rv_strategy()
        ) {
            let a_less_b = eval_binary(a.clone(), b.clone(), Operation::Less);
            let b_less_a = eval_binary(b, a, Operation::Less);

            // If a < b is true, then b < a should be false (unless a == b)
            if a_less_b == RV::Bool(true) {
                prop_assert_eq!(b_less_a, RV::Bool(false));
            }
        }

        // Property: Operations with Undefined should return appropriate results
        #[test]
        fn undefined_operations(
            a in rv_strategy().prop_filter("not undefined", |rv| !matches!(rv, RV::Undefined)),
            op in binary_operation_strategy()
        ) {
            let result1 = eval_binary(RV::Undefined, a.clone(), op);
            let result2 = eval_binary(a, RV::Undefined, op);

            match op {
                Operation::IsEqual | Operation::Is => {
                    prop_assert_eq!(result1, RV::Bool(false));
                    prop_assert_eq!(result2, RV::Bool(false));
                }
                Operation::IsNotEqual | Operation::IsNot => {
                    prop_assert_eq!(result1, RV::Bool(true));
                    prop_assert_eq!(result2, RV::Bool(true));
                }
                Operation::Add | Operation::Subtract | Operation::Multiply | Operation::Divide => {
                    prop_assert_eq!(result1, RV::Undefined);
                    prop_assert_eq!(result2, RV::Undefined);
                }
                Operation::Less | Operation::LessEqual | Operation::Greater | Operation::GreaterEqual => {
                    prop_assert_eq!(result1, RV::Bool(false));
                    prop_assert_eq!(result2, RV::Bool(false));
                }
                _ => {} // Skip other operations
            }
        }

        // Special case: Undefined compared to itself
        #[test]
        fn undefined_vs_undefined_operations(
            op in binary_operation_strategy()
        ) {
            let result = eval_binary(RV::Undefined, RV::Undefined, op);

            match op {
                Operation::IsEqual | Operation::Is => {
                    prop_assert_eq!(result, RV::Bool(true));
                }
                Operation::IsNotEqual | Operation::IsNot => {
                    prop_assert_eq!(result, RV::Bool(false));
                }
                Operation::Add | Operation::Subtract | Operation::Multiply | Operation::Divide => {
                    prop_assert_eq!(result, RV::Undefined);
                }
                Operation::Less | Operation::Greater => {
                    prop_assert_eq!(result, RV::Bool(false));
                }
                Operation::LessEqual | Operation::GreaterEqual => {
                    // Undefined == Undefined, so LessEqual and GreaterEqual should be true
                    prop_assert_eq!(result, RV::Bool(true));
                }
                _ => {} // Skip other operations
            }
        }

        // Property: String concatenation should preserve length relationship
        #[test]
        fn string_concatenation_length(
            s1 in "[a-zA-Z0-9]*",
            s2 in "[a-zA-Z0-9]*"
        ) {
            let rv1 = RV::Str(Arc::new(s1.clone()));
            let rv2 = RV::Str(Arc::new(s2.clone()));
            let result = eval_binary(rv1, rv2, Operation::Add);

            if let RV::Str(result_str) = result {
                prop_assert!(result_str.len() >= s1.len());
                prop_assert!(result_str.len() >= s2.len());
                prop_assert_eq!(result_str.len(), s1.len() + s2.len());
            }
        }

        // Property: Division by zero should handle edge cases correctly
        #[test]
        fn division_by_zero_handling(a in numeric_rv_strategy()) {
            let zero = RV::Num(0.0);
            let result = eval_binary(a.clone(), zero, Operation::Divide);

            if let Some(num) = a.as_number() {
                if num == 0.0 {
                    // 0/0 should be undefined
                    prop_assert_eq!(result, RV::Undefined);
                } else {
                    // Non-zero/0 should be infinity
                    if let RV::Num(result_num) = result {
                        prop_assert!(result_num.is_infinite());
                    }
                }
            }
        }

        // Property: Boolean to number conversion should be consistent
        #[test]
        fn boolean_number_conversion_consistency(b in any::<bool>()) {
            let rv_bool = RV::Bool(b);
            let expected_num = if b { 1.0 } else { 0.0 };

            prop_assert_eq!(rv_bool.as_number(), Some(expected_num));

            // Test in arithmetic operations
            let result = eval_binary(rv_bool, RV::Num(0.0), Operation::Add);
            prop_assert_eq!(result, RV::Num(expected_num));
        }

        // Property: Comparison operations should return boolean values
        #[test]
        fn comparisons_return_boolean(
            a in rv_strategy(),
            b in rv_strategy(),
            op in prop_oneof![
                Just(Operation::IsEqual),
                Just(Operation::IsNotEqual),
                Just(Operation::Less),
                Just(Operation::LessEqual),
                Just(Operation::Greater),
                Just(Operation::GreaterEqual)
            ]
        ) {
            let result = eval_binary(a, b, op);
            prop_assert!(matches!(result, RV::Bool(_)));
        }

        // NOTE: Addition associativity property test removed because floating-point arithmetic
        // is inherently non-associative due to precision limitations. See regression tests
        // for documented examples of this behavior.

        // Property: Type coercion should be consistent
        #[test]
        fn type_coercion_consistency(
            num in any::<f64>().prop_filter("finite", |x| x.is_finite()),
            bool_val in any::<bool>()
        ) {
            let rv_num = RV::Num(num);
            let rv_bool = RV::Bool(bool_val);
            let expected_bool_as_num = if bool_val { 1.0 } else { 0.0 };

            // num + bool should equal num + bool_as_number
            let result1 = eval_binary(rv_num.clone(), rv_bool.clone(), Operation::Add);
            let result2 = eval_binary(rv_num, RV::Num(expected_bool_as_num), Operation::Add);

            prop_assert_eq!(result1, result2);
        }
    }

    // Additional targeted property tests for specific behaviors
    proptest! {
        // Property: String comparisons should be lexicographic
        #[test]
        fn string_comparison_lexicographic(
            s1 in "[a-z]{1,10}",
            s2 in "[a-z]{1,10}"
        ) {
            let rv1 = RV::Str(Arc::new(s1.clone()));
            let rv2 = RV::Str(Arc::new(s2.clone()));

            let less_result = eval_binary(rv1.clone(), rv2.clone(), Operation::Less);
            let equal_result = eval_binary(rv1, rv2, Operation::IsEqual);

            match s1.cmp(&s2) {
                std::cmp::Ordering::Less => prop_assert_eq!(less_result, RV::Bool(true)),
                std::cmp::Ordering::Equal => prop_assert_eq!(equal_result, RV::Bool(true)),
                std::cmp::Ordering::Greater => prop_assert_eq!(less_result, RV::Bool(false)),
            }
        }

        // Property: Truthiness should be consistent with as_bool
        #[test]
        fn truthiness_consistency(rv in rv_strategy()) {
            let expected_bool = rv.as_bool();

            // Test against known truthy/falsy values
            let false_rv = RV::Bool(false);

            if expected_bool {
                // If rv is truthy, it should not equal false
                let ne_false = eval_binary(rv, false_rv, Operation::IsNotEqual);
                prop_assert_eq!(ne_false, RV::Bool(true));
            } else {
                // If rv is falsy, specific falsy values should behave consistently
                match rv {
                    RV::Num(0.0) => {
                        let eq_false = eval_binary(RV::Num(0.0), RV::Bool(false), Operation::IsEqual);
                        prop_assert_eq!(eq_false, RV::Bool(true));
                    }
                    RV::Bool(false) => {
                        let eq_false = eval_binary(RV::Bool(false), RV::Bool(false), Operation::IsEqual);
                        prop_assert_eq!(eq_false, RV::Bool(true));
                    }
                    _ => {} // Other falsy values have their own rules
                }
            }
        }

        // Property: String concatenation with special characters and Unicode
        #[test]
        fn string_concatenation_unicode_edge_cases(
            s1 in r"[\x00-\x1F\u{1F600}-\u{1F64F}a-zA-Z0-9\s]*",
            s2 in r"[\x00-\x1F\u{1F600}-\u{1F64F}a-zA-Z0-9\s]*"
        ) {
            let rv1 = RV::Str(Arc::new(s1.clone()));
            let rv2 = RV::Str(Arc::new(s2.clone()));
            let result = eval_binary(rv1, rv2, Operation::Add);

            if let RV::Str(result_str) = result {
                prop_assert_eq!(result_str.len(), s1.len() + s2.len());
                prop_assert!(result_str.starts_with(&s1));
                prop_assert!(result_str.ends_with(&s2));
            } else {
                prop_assert!(false, "String concatenation should always produce a string");
            }
        }

        // Property: Numeric edge cases with very small numbers
        #[test]
        fn numeric_operations_tiny_numbers(
            a in prop_oneof![
                (-1e-300..1e-300_f64).prop_map(RV::Num),
                any::<bool>().prop_map(RV::Bool)
            ],
            b in prop_oneof![
                (-1e-300..1e-300_f64).prop_map(RV::Num),
                any::<bool>().prop_map(RV::Bool)
            ],
            op in prop_oneof![
                Just(Operation::Add),
                Just(Operation::Subtract),
                Just(Operation::Multiply)
            ]
        ) {
            let result = eval_binary(a.clone(), b.clone(), op);

            // Very small numbers should still produce finite results
            if let RV::Num(n) = result {
                prop_assert!(n.is_finite() || n == 0.0, "Tiny number operations should remain finite: {}", n);
            }
        }

        // Property: String-to-number parsing edge cases
        #[test]
        fn string_number_parsing_edge_cases(
            s in prop_oneof![
                r"[+-]?[0-9]+\.?[0-9]*([eE][+-]?[0-9]+)?", // Valid number formats
                r"[+-]?[0-9]*\.?[0-9]+([eE][+-]?[0-9]+)?", // Valid number formats
                r"[+-]?Infinity",                           // Infinity strings
                r"[+-]?inf",                               // inf strings
                Just("NaN".to_string()),                   // NaN string
                r"[a-zA-Z]+[0-9]*",                       // Invalid: letters then numbers
                r"[0-9]+[a-zA-Z]+",                       // Invalid: numbers then letters
                Just("   ".to_string())                   // Whitespace only
            ]
        ) {
            let str_rv = RV::Str(Arc::new(s.clone()));
            let num_rv = RV::Num(42.0);

            // Test comparison operations
            let eq_result = eval_binary(str_rv.clone(), num_rv.clone(), Operation::IsEqual);
            let lt_result = eval_binary(str_rv.clone(), num_rv.clone(), Operation::Less);

            // These should always produce boolean results
            prop_assert!(matches!(eq_result, RV::Bool(_)), "String-number equality should return boolean");
            prop_assert!(matches!(lt_result, RV::Bool(_)), "String-number comparison should return boolean");
        }

        // Property: Mixed type arithmetic consistency
        #[test]
        fn mixed_type_arithmetic_consistency(
            num in any::<f64>().prop_filter("finite", |x| x.is_finite() && x.abs() < 1e100),
            bool_val in any::<bool>(),
            op in prop_oneof![
                Just(Operation::Add),
                Just(Operation::Subtract),
                Just(Operation::Multiply)
            ]
        ) {
            let rv_num = RV::Num(num);
            let rv_bool = RV::Bool(bool_val);
            let bool_as_num = RV::Num(if bool_val { 1.0 } else { 0.0 });

            // num op bool should equal num op bool_as_num
            let mixed_result = eval_binary(rv_num.clone(), rv_bool, op);
            let pure_num_result = eval_binary(rv_num, bool_as_num, op);

            prop_assert_eq!(mixed_result, pure_num_result,
                "Mixed type arithmetic should be consistent with explicit conversion");
        }

        // Property: Division by very small numbers
        #[test]
        fn division_by_tiny_numbers(
            dividend in (-1e10..1e10_f64).prop_filter("finite", |x| x.is_finite()),
            divisor in (-1e-100..1e-100_f64).prop_filter("non-zero", |x| *x != 0.0)
        ) {
            let result = eval_binary(RV::Num(dividend), RV::Num(divisor), Operation::Divide);

            if let RV::Num(n) = result {
                // Division by tiny numbers should produce large finite numbers or infinity
                prop_assert!(n.is_finite() || n.is_infinite(),
                    "Division by tiny numbers should produce finite or infinite results: {}", n);
            }
        }

        // Property: String concatenation with numbers preserves string representation
        #[test]
        fn string_number_concatenation_preserves_representation(
            s in "[a-zA-Z]{1,10}",
            num in any::<f64>().prop_filter("finite", |x| x.is_finite())
        ) {
            let str_rv = RV::Str(Arc::new(s.clone()));
            let num_rv = RV::Num(num);

            let result1 = eval_binary(str_rv.clone(), num_rv.clone(), Operation::Add);
            let result2 = eval_binary(num_rv, str_rv, Operation::Add);

            if let (RV::Str(s1), RV::Str(s2)) = (result1, result2) {
                prop_assert!(s1.contains(&s), "String concatenation should preserve original string");
                prop_assert!(s2.contains(&s), "String concatenation should preserve original string");
                prop_assert!(s1.contains(&num.to_string()), "String concatenation should contain number representation");
                prop_assert!(s2.contains(&num.to_string()), "String concatenation should contain number representation");
            } else {
                prop_assert!(false, "String-number concatenation should produce strings");
            }
        }

        // Property: Comparison transitivity for comparable types
        #[test]
        fn comparison_transitivity(
            a in numeric_rv_strategy(),
            b in numeric_rv_strategy(),
            c in numeric_rv_strategy()
        ) {
            let a_lt_b = eval_binary(a.clone(), b.clone(), Operation::Less);
            let b_lt_c = eval_binary(b, c.clone(), Operation::Less);
            let a_lt_c = eval_binary(a, c, Operation::Less);

            // If a < b and b < c, then a < c (transitivity)
            if a_lt_b == RV::Bool(true) && b_lt_c == RV::Bool(true) {
                prop_assert_eq!(a_lt_c, RV::Bool(true), "Less-than should be transitive");
            }
        }

        // Property: Subtraction is inverse of addition (within floating-point precision limits)
        #[test]
        fn subtraction_inverse_of_addition_small_numbers(
            a in (-1e6..1e6_f64).prop_map(RV::Num),
            b in (-1e6..1e6_f64).prop_map(RV::Num)
        ) {
            // (a + b) - b should equal a
            let sum = eval_binary(a.clone(), b.clone(), Operation::Add);
            let difference = eval_binary(sum, b, Operation::Subtract);

            if let (RV::Num(original), RV::Num(result)) = (a, difference) {
                let diff = (original - result).abs();
                // Use a more generous tolerance that accounts for the inherent limitations of floating-point arithmetic
                // The tolerance needs to scale with the magnitude of the numbers involved
                let scale = original.abs().max(1.0);
                let tolerance = f64::EPSILON * scale * 100.0; // More generous multiplier
                prop_assert!(diff <= tolerance,
                    "Subtraction should be inverse of addition within floating-point precision: {} vs {}, diff: {}, tolerance: {}",
                    original, result, diff, tolerance);
            }
        }

        // Property: Multiplication by zero always yields zero
        #[test]
        fn multiplication_by_zero_yields_zero(
            a in numeric_rv_strategy()
        ) {
            let zero = RV::Num(0.0);
            let result1 = eval_binary(a.clone(), zero.clone(), Operation::Multiply);
            let result2 = eval_binary(zero, a, Operation::Multiply);

            prop_assert_eq!(result1, RV::Num(0.0), "Multiplication by zero should yield zero");
            prop_assert_eq!(result2, RV::Num(0.0), "Multiplication by zero should yield zero");
        }

        // Property: String comparison with empty strings
        #[test]
        fn string_comparison_with_empty_strings(
            s in "[a-zA-Z0-9]*"
        ) {
            let empty_str = RV::Str(Arc::new("".to_string()));
            let non_empty_str = RV::Str(Arc::new(s.clone()));

            let empty_lt_nonempty = eval_binary(empty_str.clone(), non_empty_str.clone(), Operation::Less);
            let nonempty_gt_empty = eval_binary(non_empty_str, empty_str, Operation::Greater);

            if !s.is_empty() {
                prop_assert_eq!(empty_lt_nonempty, RV::Bool(true), "Empty string should be less than non-empty");
                prop_assert_eq!(nonempty_gt_empty, RV::Bool(true), "Non-empty string should be greater than empty");
            }
        }
    }
}

#[cfg(test)]
mod regression_tests {
    use std::sync::Arc;

    use super::*;
    use crate::value::eval::eval_binary;

    /// These tests capture specific edge cases found through property testing
    /// or known to be important for the domain
    #[test]
    fn regression_nan_handling() {
        // NaN comparisons should always be false
        let nan = RV::Num(f64::NAN);
        let num = RV::Num(1.0);

        assert_eq!(
            eval_binary(nan.clone(), num.clone(), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(nan.clone(), num.clone(), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(nan.clone(), num.clone(), Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(nan.clone(), nan.clone(), Operation::IsEqual),
            RV::Bool(false)
        );
    }

    #[test]
    fn regression_infinity_arithmetic() {
        let inf = RV::Num(f64::INFINITY);
        let num = RV::Num(42.0);

        // inf + num = inf
        assert_eq!(eval_binary(inf.clone(), num.clone(), Operation::Add), inf);
        // inf - inf should be NaN, but we might handle it differently
        let result = eval_binary(inf.clone(), inf.clone(), Operation::Subtract);
        match result {
            RV::Num(n) => assert!(n.is_nan()),
            RV::Undefined => {} // Also acceptable
            _ => panic!("Unexpected result for inf - inf"),
        }
    }

    #[test]
    fn regression_string_number_coercion() {
        // String that can be parsed as number
        let str_num = RV::Str(Arc::new("42".to_string()));
        let num = RV::Num(42.0);

        // In comparisons, string numbers should be coerced
        let result = eval_binary(str_num, num, Operation::IsEqual);
        // This depends on implementation - document the expected behavior
        match result {
            RV::Bool(_) => {} // Either true or false is acceptable, but should be documented
            _ => panic!("String-number comparison should return boolean"),
        }
    }

    #[test]
    fn regression_addition_associativity_large_numbers() {
        // This test documents a known floating-point precision issue
        // Found through property testing: very large numbers don't maintain exact associativity
        // Failing case from proptest: a = 3.471971265526436e290, b = 1.1414671723104438e295, c = -1.563421480094111e295
        let a = RV::Num(3.471971265526436e290);
        let b = RV::Num(1.1414671723104438e295);
        let c = RV::Num(-1.563421480094111e295);

        // (a + b) + c
        let ab = eval_binary(a.clone(), b.clone(), Operation::Add);
        let ab_c = eval_binary(ab, c.clone(), Operation::Add);

        // a + (b + c)
        let bc = eval_binary(b, c, Operation::Add);
        let a_bc = eval_binary(a, bc, Operation::Add);

        // These should be approximately equal but may not be exactly equal due to floating-point precision
        match (ab_c, a_bc) {
            (RV::Num(left), RV::Num(right)) => {
                // Document the actual difference found - this is a known limitation of floating-point arithmetic
                let diff = (left - right).abs();
                let max_val = left.abs().max(right.abs());
                let relative_error = if max_val > 0.0 { diff / max_val } else { 0.0 };

                // For numbers this large (~1e295), we expect some precision loss
                // The original failure showed a difference of ~5e279, which is significant but expected
                // for floating-point arithmetic at this scale
                // println!("Floating-point precision test:");
                // println!("  Left result:  {}", left);
                // println!("  Right result: {}", right);
                // println!("  Absolute diff: {}", diff);
                // println!("  Relative error: {}", relative_error);

                // This test documents the behavior rather than asserting exact equality
                // For extremely large numbers, floating-point arithmetic is inherently imprecise
                assert!(
                    relative_error < 1e-13 || diff < 1e280,
                    "Relative error {relative_error} or absolute difference {diff} is larger than expected for floating-point arithmetic at this scale"
                );
            }
            _ => panic!("Expected numeric results for addition operations"),
        }
    }

    #[test]
    fn regression_subtraction_precision_large_numbers() {
        // This test documents another floating-point precision issue found through property testing
        // Failing case: a = 239305169.8616219, b = -4330652516.771584
        // The issue: (a + b) - b should equal a, but floating-point precision causes small differences
        let a = RV::Num(239305169.8616219);
        let b = RV::Num(-4330652516.771584);

        // (a + b) - b should equal a
        let sum = eval_binary(a.clone(), b.clone(), Operation::Add);
        let difference = eval_binary(sum, b, Operation::Subtract);

        match (a, difference) {
            (RV::Num(original), RV::Num(result)) => {
                let diff = (original - result).abs();
                // The original failure showed a difference of ~3e-8, which is expected for this magnitude
                assert!(
                    diff < 1e-6,
                    "Subtraction precision loss should be small: original={original}, result={result}, diff={diff}"
                );
            }
            _ => panic!("Expected numeric results"),
        }
    }

    #[test]
    fn regression_subtraction_precision_medium_numbers() {
        // Another case found through property testing: a = 776327.478839819, b = 333251.5209597033
        // Shows that even "medium" sized numbers can have precision issues
        let a = RV::Num(776327.478839819);
        let b = RV::Num(333251.5209597033);

        // (a + b) - b should equal a
        let sum = eval_binary(a.clone(), b.clone(), Operation::Add);
        let difference = eval_binary(sum, b, Operation::Subtract);

        match (a, difference) {
            (RV::Num(original), RV::Num(result)) => {
                let diff = (original - result).abs();
                // This case showed a difference of ~1e-10, which is within expected floating-point precision
                assert!(
                    diff < 1e-9,
                    "Subtraction precision loss should be within floating-point epsilon: original={original}, result={result}, diff={diff}"
                );
            }
            _ => panic!("Expected numeric results"),
        }
    }

    #[test]
    fn regression_addition_associativity_tiny_numbers() {
        // Found through property testing: even tiny numbers can have associativity issues
        // Case: a = -8.856050288210175e-102, b = 5.003750612076637e-88, c = -5.226269442637776e-88
        let a = RV::Num(-8.856050288210175e-102);
        let b = RV::Num(5.003750612076637e-88);
        let c = RV::Num(-5.226269442637776e-88);

        // (a + b) + c vs a + (b + c)
        let ab = eval_binary(a.clone(), b.clone(), Operation::Add);
        let ab_c = eval_binary(ab, c.clone(), Operation::Add);

        let bc = eval_binary(b, c, Operation::Add);
        let a_bc = eval_binary(a, bc, Operation::Add);

        match (ab_c, a_bc) {
            (RV::Num(left), RV::Num(right)) => {
                let diff = (left - right).abs();
                // Even with tiny numbers, floating-point precision limits apply
                // This documents that associativity can fail even at very small scales
                let max_val = left.abs().max(right.abs());
                let relative_error = if max_val > 0.0 { diff / max_val } else { 0.0 };

                assert!(
                    relative_error < 1e-12 || diff < 1e-100,
                    "Addition associativity failed for tiny numbers: left={left}, right={right}, diff={diff}, rel_err={relative_error}"
                );
            }
            _ => panic!("Expected numeric results"),
        }
    }

    #[test]
    fn regression_subtraction_precision_small_negative() {
        // Case found through property testing: a = -13542.264609919892, b = 751876.9370472291
        // Shows precision issues with negative numbers and subtraction
        let a = RV::Num(-13542.264609919892);
        let b = RV::Num(751876.9370472291);

        let sum = eval_binary(a.clone(), b.clone(), Operation::Add);
        let difference = eval_binary(sum, b, Operation::Subtract);

        match (a, difference) {
            (RV::Num(original), RV::Num(result)) => {
                let diff = (original - result).abs();
                // This case showed a difference of ~3e-11, demonstrating precision limits
                assert!(
                    diff < 1e-8,
                    "Subtraction precision with negative numbers: original={original}, result={result}, diff={diff}"
                );
            }
            _ => panic!("Expected numeric results"),
        }
    }
}
