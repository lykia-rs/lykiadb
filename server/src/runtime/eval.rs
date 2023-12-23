use std::rc::Rc;

use crate::lang::ast::expr::Operation;
use crate::runtime::types::RV;

#[macro_export]
macro_rules! bool2num {
    ($val: expr) => {
        if $val {
            1.0
        } else {
            0.0
        }
    };
}
#[inline(always)]
pub fn is_value_truthy(rv: RV) -> bool {
    match rv {
        RV::Num(value) => !value.is_nan() && value.abs() > 0.0,
        RV::Str(value) => !value.is_empty(),
        RV::Bool(value) => value,
        RV::Null | RV::Undefined | RV::NaN => false,
        _ => true,
    }
}

#[inline(always)]
pub fn coerce2number(val: RV) -> Option<f64> {
    match val {
        RV::Num(value) => Some(value),
        RV::Bool(true) => Some(1.0),
        RV::Bool(false) => Some(0.0),
        _ => None,
    }
}

#[inline(always)]
pub fn eval_binary(left_eval: RV, right_eval: RV, operation: Operation) -> RV {
    /*
        TODO(vck):
            - Add support for object operations
            - Add support for array operations
            - Add support for function operations
    */
    let (left_coerced, right_coerced) = match (&left_eval, &operation, &right_eval) {
        (RV::Num(n), _, RV::Bool(bool)) => (RV::Num(*n), RV::Num(bool2num!(*bool))),
        (RV::Bool(bool), _, RV::Num(n)) => (RV::Num(bool2num!(*bool)), RV::Num(*n)),
        (RV::Bool(l), Operation::Add, RV::Bool(r))
        | (RV::Bool(l), Operation::Subtract, RV::Bool(r))
        | (RV::Bool(l), Operation::Multiply, RV::Bool(r))
        | (RV::Bool(l), Operation::Divide, RV::Bool(r)) => {
            (RV::Num(bool2num!(*l)), RV::Num(bool2num!(*r)))
        }
        (_, _, _) => (left_eval, right_eval),
    };

    match (left_coerced, operation, right_coerced) {
        (RV::Null, Operation::IsEqual, RV::Null) => RV::Bool(true),
        (RV::Null, Operation::IsNotEqual, RV::Null) => RV::Bool(false),
        //
        (_, Operation::IsEqual, RV::Null) | (RV::Null, Operation::IsEqual, _) => RV::Bool(false),
        //
        (RV::NaN, Operation::Add, _)
        | (_, Operation::Add, RV::NaN)
        | (RV::NaN, Operation::Subtract, _)
        | (_, Operation::Subtract, RV::NaN)
        | (RV::NaN, Operation::Multiply, _)
        | (_, Operation::Multiply, RV::NaN)
        | (RV::NaN, Operation::Divide, _)
        | (_, Operation::Divide, RV::NaN) => RV::NaN,
        //
        (RV::Num(l), Operation::Add, RV::Num(r)) => RV::Num(l + r),
        (RV::Num(l), Operation::Subtract, RV::Num(r)) => RV::Num(l - r),
        (RV::Num(l), Operation::Multiply, RV::Num(r)) => RV::Num(l * r),
        (RV::Num(l), Operation::Divide, RV::Num(r)) => {
            if l == 0.0 && r == 0.0 {
                RV::NaN
            } else {
                RV::Num(l / r)
            }
        }
        (RV::Num(l), Operation::Less, RV::Num(r)) => RV::Bool(l < r),
        (RV::Num(l), Operation::LessEqual, RV::Num(r)) => RV::Bool(l <= r),
        (RV::Num(l), Operation::Greater, RV::Num(r)) => RV::Bool(l > r),
        (RV::Num(l), Operation::GreaterEqual, RV::Num(r)) => RV::Bool(l >= r),
        (RV::Num(l), Operation::IsNotEqual, RV::Num(r)) => RV::Bool(l != r),
        (RV::Num(l), Operation::IsEqual, RV::Num(r)) => RV::Bool(l == r),
        //
        (RV::Str(l), Operation::Add, RV::Str(r)) => {
            RV::Str(Rc::new(l.to_string() + &r.to_string()))
        }
        (RV::Str(l), Operation::Less, RV::Str(r)) => RV::Bool(l < r),
        (RV::Str(l), Operation::LessEqual, RV::Str(r)) => RV::Bool(l <= r),
        (RV::Str(l), Operation::Greater, RV::Str(r)) => RV::Bool(l > r),
        (RV::Str(l), Operation::GreaterEqual, RV::Str(r)) => RV::Bool(l >= r),
        (RV::Str(l), Operation::IsNotEqual, RV::Str(r)) => RV::Bool(l != r),
        (RV::Str(l), Operation::IsEqual, RV::Str(r)) => RV::Bool(l == r),
        //
        (RV::Bool(l), Operation::Less, RV::Bool(r)) => RV::Bool(!l & r),
        (RV::Bool(l), Operation::LessEqual, RV::Bool(r)) => RV::Bool(l <= r),
        (RV::Bool(l), Operation::Greater, RV::Bool(r)) => RV::Bool(l & !r),
        (RV::Bool(l), Operation::GreaterEqual, RV::Bool(r)) => RV::Bool(l >= r),
        (RV::Bool(l), Operation::IsNotEqual, RV::Bool(r)) => RV::Bool(l != r),
        (RV::Bool(l), Operation::IsEqual, RV::Bool(r)) => RV::Bool(l == r),
        //
        (RV::Str(s), Operation::Add, RV::Num(num)) => {
            RV::Str(Rc::new(s.to_string() + &num.to_string()))
        }
        (RV::Num(num), Operation::Add, RV::Str(s)) => {
            RV::Str(Rc::new(num.to_string() + &s.to_string()))
        }
        //
        (RV::Str(s), Operation::Add, RV::Bool(bool)) => {
            RV::Str(Rc::new(s.to_string() + &bool.to_string()))
        }
        (RV::Bool(bool), Operation::Add, RV::Str(s)) => {
            RV::Str(Rc::new(bool.to_string() + &s.to_string()))
        }
        //
        (_, Operation::Less, _)
        | (_, Operation::LessEqual, _)
        | (_, Operation::Greater, _)
        | (_, Operation::GreaterEqual, _)
        | (_, Operation::IsEqual, _) => RV::Bool(false),
        //
        (_, Operation::IsNotEqual, _) => RV::Bool(true),
        //
        (_, Operation::Add, _)
        | (_, Operation::Subtract, _)
        | (_, Operation::Multiply, _)
        | (_, Operation::Divide, _) => RV::NaN,
        //
        (_, _, _) => RV::Undefined,
    }
}

#[cfg(test)]
mod test {
    use std::{f64::INFINITY, rc::Rc};

    use rustc_hash::FxHashMap;

    use crate::{
        lang::ast::expr::Operation,
        runtime::{
            eval::{coerce2number, eval_binary, is_value_truthy},
            types::{Function, RV},
        },
        util::alloc_shared,
    };

    #[test]
    fn test_is_value_truthy() {
        assert_eq!(is_value_truthy(RV::Null), false);
        assert_eq!(is_value_truthy(RV::Undefined), false);
        assert_eq!(is_value_truthy(RV::NaN), false);
        assert_eq!(is_value_truthy(RV::Bool(false)), false);
        assert_eq!(is_value_truthy(RV::Bool(true)), true);
        assert_eq!(is_value_truthy(RV::Num(0.0)), false);
        assert_eq!(is_value_truthy(RV::Num(0.1)), true);
        assert_eq!(is_value_truthy(RV::Num(1.0)), true);
        assert_eq!(is_value_truthy(RV::Num(0.0)), false);
        assert_eq!(is_value_truthy(RV::Num(-1.0)), true);
        assert_eq!(is_value_truthy(RV::Str(Rc::new("".to_owned()))), false);
        assert_eq!(is_value_truthy(RV::Str(Rc::new("foo".to_owned()))), true);
        assert_eq!(is_value_truthy(RV::Array(alloc_shared(vec![]))), true);
        assert_eq!(
            is_value_truthy(RV::Object(alloc_shared(FxHashMap::default()))),
            true
        );
        assert_eq!(
            is_value_truthy(RV::Callable(
                Some(1),
                Rc::new(Function::Lambda {
                    function: |_, _| Ok(RV::Undefined)
                })
            )),
            true
        );
    }

    #[test]
    fn test_coerce2number() {
        assert_eq!(coerce2number(RV::Num(1.0)), Some(1.0));
        assert_eq!(coerce2number(RV::Bool(false)), Some(0.0));
        assert_eq!(coerce2number(RV::Bool(true)), Some(1.0));
        assert_eq!(coerce2number(RV::Str(Rc::new("".to_owned()))), None);
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
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("b".to_string())),
                Operation::Add
            ),
            RV::Str(Rc::new("ab".to_string()))
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("b".to_string())),
                RV::Str(Rc::new("a".to_string())),
                Operation::Add
            ),
            RV::Str(Rc::new("ba".to_string()))
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
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("b".to_string())),
                Operation::Subtract
            ),
            RV::NaN
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("b".to_string())),
                RV::Str(Rc::new("a".to_string())),
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
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("b".to_string())),
                Operation::Multiply
            ),
            RV::NaN
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("b".to_string())),
                RV::Str(Rc::new("a".to_string())),
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
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("b".to_string())),
                Operation::Divide
            ),
            RV::NaN
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("b".to_string())),
                RV::Str(Rc::new("a".to_string())),
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
                RV::Str(Rc::new("a".to_string())),
                Operation::IsEqual
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Num(1.0),
                Operation::IsEqual
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("b".to_string())),
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
                RV::Str(Rc::new("a".to_string())),
                Operation::IsNotEqual
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Num(1.0),
                Operation::IsNotEqual
            ),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("b".to_string())),
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
                RV::Str(Rc::new("a".to_string())),
                Operation::Less
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Num(1.0),
                Operation::Less
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("a".to_string())),
                Operation::Less
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("b".to_string())),
                Operation::Less
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("b".to_string())),
                RV::Str(Rc::new("a".to_string())),
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
                RV::Str(Rc::new("a".to_string())),
                Operation::LessEqual
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Num(1.0),
                Operation::LessEqual
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("a".to_string())),
                Operation::LessEqual
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("b".to_string())),
                Operation::LessEqual
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("b".to_string())),
                RV::Str(Rc::new("a".to_string())),
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
                RV::Str(Rc::new("a".to_string())),
                Operation::Greater
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Num(1.0),
                Operation::Greater
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("a".to_string())),
                Operation::Greater
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("b".to_string())),
                Operation::Greater
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("b".to_string())),
                RV::Str(Rc::new("a".to_string())),
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
                RV::Str(Rc::new("a".to_string())),
                Operation::GreaterEqual
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Num(1.0),
                Operation::GreaterEqual
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("a".to_string())),
                Operation::GreaterEqual
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
                RV::Str(Rc::new("b".to_string())),
                Operation::GreaterEqual
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("b".to_string())),
                RV::Str(Rc::new("a".to_string())),
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
                RV::Str(Rc::new("a".to_string())),
                RV::Bool(true),
                Operation::Add
            ),
            RV::Str(Rc::new("atrue".to_string()))
        );
        assert_eq!(
            eval_binary(
                RV::Str(Rc::new("a".to_string())),
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
                RV::Str(Rc::new("a".to_string())),
                Operation::Add
            ),
            RV::Str(Rc::new("truea".to_string()))
        );
        assert_eq!(
            eval_binary(
                RV::Bool(true),
                RV::Str(Rc::new("a".to_string())),
                Operation::Less
            ),
            RV::Bool(false)
        );
    }
}
