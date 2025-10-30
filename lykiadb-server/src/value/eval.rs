use super::StdVal;
use lykiadb_lang::ast::expr::Operation;
use std::ops;
use std::sync::Arc;

impl PartialEq for StdVal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StdVal::Array(_), StdVal::Array(_)) | (StdVal::Object(_), StdVal::Object(_)) => false,
            (StdVal::Undefined, StdVal::Undefined) => true,
            //
            (StdVal::Undefined, _) | (_, StdVal::Undefined) => false,
            //
            (StdVal::Str(a), StdVal::Str(b)) => a == b,
            (StdVal::Num(a), StdVal::Num(b)) => a == b,
            (StdVal::Bool(a), StdVal::Bool(b)) => a == b,
            //
            (StdVal::Str(_), StdVal::Num(b)) => self.eq_str_num(*b),
            (StdVal::Num(a), StdVal::Str(_)) => other.eq_str_num(*a),
            //
            (StdVal::Str(_), StdVal::Bool(b)) => self.eq_any_bool(*b),
            (StdVal::Bool(a), StdVal::Str(_)) => other.eq_any_bool(*a),
            //
            (StdVal::Num(_), StdVal::Bool(b)) => self.eq_any_bool(*b),
            (StdVal::Bool(a), StdVal::Num(_)) => other.eq_any_bool(*a),
            //
            (StdVal::Datatype(a), StdVal::Datatype(b)) => a == b,
            //
            _ => false,
        }
    }
}

impl PartialOrd for StdVal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (StdVal::Array(_), StdVal::Array(_)) | (StdVal::Object(_), StdVal::Object(_)) => None,
            (StdVal::Undefined, StdVal::Undefined) => Some(std::cmp::Ordering::Equal),
            //
            (StdVal::Undefined, _) | (_, StdVal::Undefined) => None,
            //
            (StdVal::Str(a), StdVal::Str(b)) => Some(a.cmp(b)),
            (StdVal::Num(a), StdVal::Num(b)) => a.partial_cmp(b),
            (StdVal::Bool(a), StdVal::Bool(b)) => a.partial_cmp(b),
            //
            (StdVal::Str(a), StdVal::Num(b)) => {
                if let Ok(num) = a.parse::<f64>() {
                    return num.partial_cmp(b);
                }
                None
            }
            (StdVal::Num(a), StdVal::Str(b)) => {
                if let Ok(num) = b.parse::<f64>() {
                    return a.partial_cmp(&num);
                }
                None
            }
            //
            (StdVal::Str(_), StdVal::Bool(b)) => self.partial_cmp_str_bool(*b),
            (StdVal::Bool(a), StdVal::Str(_)) => other.partial_cmp_str_bool(*a),
            //
            (StdVal::Num(num), StdVal::Bool(b)) => num.partial_cmp(&if *b { 1.0 } else { 0.0 }),
            (StdVal::Bool(b), StdVal::Num(num)) => (if *b { 1.0 } else { 0.0 }).partial_cmp(num),
            //
            _ => None,
        }
    }
}

impl ops::Add for StdVal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            //
            (StdVal::Bool(_), StdVal::Bool(_)) | (StdVal::Num(_), StdVal::Bool(_)) | (StdVal::Bool(_), StdVal::Num(_)) => {
                StdVal::Num(self.as_number().unwrap() + rhs.as_number().unwrap())
            }

            (StdVal::Num(l), StdVal::Num(r)) => StdVal::Num(l + r),
            //
            (StdVal::Str(l), StdVal::Str(r)) => StdVal::Str(Arc::new(l.to_string() + &r.to_string())),
            //
            (StdVal::Str(s), StdVal::Num(num)) => StdVal::Str(Arc::new(s.to_string() + &num.to_string())),
            (StdVal::Num(num), StdVal::Str(s)) => StdVal::Str(Arc::new(num.to_string() + &s.to_string())),
            //
            (StdVal::Str(s), StdVal::Bool(bool)) => StdVal::Str(Arc::new(s.to_string() + &bool.to_string())),
            (StdVal::Bool(bool), StdVal::Str(s)) => StdVal::Str(Arc::new(bool.to_string() + &s.to_string())),
            //
            (_, _) => StdVal::Undefined,
        }
    }
}

impl ops::Sub for StdVal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (StdVal::Undefined, _) | (_, StdVal::Undefined) => StdVal::Undefined,
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| StdVal::Num(a - b))
                .unwrap_or(StdVal::Undefined),
        }
    }
}

impl ops::Mul for StdVal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (StdVal::Undefined, _) | (_, StdVal::Undefined) => StdVal::Undefined,
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| StdVal::Num(a * b))
                .unwrap_or(StdVal::Undefined),
        }
    }
}

impl ops::Div for StdVal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (StdVal::Undefined, _) | (_, StdVal::Undefined) => StdVal::Undefined,
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| {
                    if a == 0.0 && b == 0.0 {
                        StdVal::Undefined
                    } else {
                        StdVal::Num(a / b)
                    }
                })
                .unwrap_or(StdVal::Undefined),
        }
    }
}

#[inline(always)]
pub fn eval_binary(left_eval: StdVal, right_eval: StdVal, operation: Operation) -> StdVal {
    /*
        TODO(vck):
            - Add support for object operations
            - Add support for array operations
            - Add support for function operations
    */
    match operation {
        Operation::Is | Operation::IsEqual => StdVal::Bool(left_eval == right_eval),
        Operation::IsNot | Operation::IsNotEqual => StdVal::Bool(left_eval != right_eval),
        Operation::Less => StdVal::Bool(left_eval < right_eval),
        Operation::LessEqual => StdVal::Bool(left_eval <= right_eval),
        Operation::Greater => StdVal::Bool(left_eval > right_eval),
        Operation::GreaterEqual => StdVal::Bool(left_eval >= right_eval),
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
        _ => StdVal::Undefined,
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use lykiadb_lang::ast::expr::Operation;
    use rustc_hash::FxHashMap;

    use crate::{
        util::alloc_shared,
        value::eval::{StdVal, eval_binary},
    };

    #[test]
    fn test_is_value_truthy() {
        assert!(!(StdVal::Undefined).as_bool());
        assert!(!(StdVal::Bool(false)).as_bool());
        assert!((StdVal::Bool(true)).as_bool());
        assert!(!(StdVal::Num(0.0)).as_bool());
        assert!((StdVal::Num(0.1)).as_bool());
        assert!((StdVal::Num(-0.1)).as_bool());
        assert!((StdVal::Num(1.0)).as_bool());
        assert!((StdVal::Num(-1.0)).as_bool());
        assert!(!(StdVal::Str(Arc::new("".to_owned()))).as_bool());
        assert!((StdVal::Str(Arc::new("0".to_owned()))).as_bool());
        assert!((StdVal::Str(Arc::new("false".to_owned()))).as_bool());
        assert!((StdVal::Str(Arc::new("true".to_owned()))).as_bool());
        assert!((StdVal::Str(Arc::new("foo".to_owned()))).as_bool());
        assert!((StdVal::Array(alloc_shared(vec![]))).as_bool());
        assert!((StdVal::Object(alloc_shared(FxHashMap::default()))).as_bool());
    }

    #[test]
    fn test_as_number() {
        assert_eq!((StdVal::Num(1.0)).as_number(), Some(1.0));
        assert_eq!((StdVal::Bool(false)).as_number(), Some(0.0));
        assert_eq!((StdVal::Bool(true)).as_number(), Some(1.0));
        assert_eq!((StdVal::Str(Arc::new("".to_owned()))).as_number(), None);
    }

    #[test]
    fn test_eval_binary_addition() {
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(2.0), Operation::Add),
            StdVal::Num(3.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(2.0), StdVal::Num(1.0), Operation::Add),
            StdVal::Num(3.0)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("b".to_string())),
                Operation::Add
            ),
            StdVal::Str(Arc::new("ab".to_string()))
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("b".to_string())),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::Add
            ),
            StdVal::Str(Arc::new("ba".to_string()))
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(true), Operation::Add),
            StdVal::Num(2.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(false), Operation::Add),
            StdVal::Num(1.0)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(true), Operation::Add),
            StdVal::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(false), Operation::Add),
            StdVal::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::Add),
            StdVal::Num(2.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Add),
            StdVal::Num(2.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Num(1.0), Operation::Add),
            StdVal::Num(1.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(false), Operation::Add),
            StdVal::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Add),
            StdVal::Num(2.0)
        );
    }

    #[test]
    fn test_eval_binary_subtraction() {
        assert_eq!(
            eval_binary(StdVal::Num(2.0), StdVal::Num(1.0), Operation::Subtract),
            StdVal::Num(1.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(2.0), Operation::Subtract),
            StdVal::Num(-1.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(true), Operation::Subtract),
            StdVal::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(false), Operation::Subtract),
            StdVal::Num(1.0)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(true), Operation::Subtract),
            StdVal::Num(-1.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(false), Operation::Subtract),
            StdVal::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::Subtract),
            StdVal::Num(0.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Subtract),
            StdVal::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Num(1.0), Operation::Subtract),
            StdVal::Num(-1.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(false), Operation::Subtract),
            StdVal::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Subtract),
            StdVal::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("b".to_string())),
                Operation::Subtract
            ),
            StdVal::Undefined
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("b".to_string())),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::Subtract
            ),
            StdVal::Undefined
        );
    }

    #[test]
    fn test_eval_binary_multiplication() {
        assert_eq!(
            eval_binary(StdVal::Num(2.0), StdVal::Num(1.0), Operation::Multiply),
            StdVal::Num(2.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(2.0), Operation::Multiply),
            StdVal::Num(2.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(true), Operation::Multiply),
            StdVal::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(false), Operation::Multiply),
            StdVal::Num(0.0)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(true), Operation::Multiply),
            StdVal::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(false), Operation::Multiply),
            StdVal::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::Multiply),
            StdVal::Num(1.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Multiply),
            StdVal::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Num(1.0), Operation::Multiply),
            StdVal::Num(0.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(false), Operation::Multiply),
            StdVal::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Multiply),
            StdVal::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("b".to_string())),
                Operation::Multiply
            ),
            StdVal::Undefined
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("b".to_string())),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::Multiply
            ),
            StdVal::Undefined
        );
    }

    #[test]
    fn test_eval_binary_division() {
        assert_eq!(
            eval_binary(StdVal::Num(2.0), StdVal::Num(1.0), Operation::Divide),
            StdVal::Num(2.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(2.0), Operation::Divide),
            StdVal::Num(0.5)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(true), Operation::Divide),
            StdVal::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(false), Operation::Divide),
            StdVal::Num(f64::INFINITY)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(true), Operation::Divide),
            StdVal::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(false), Operation::Divide),
            StdVal::Undefined
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::Divide),
            StdVal::Num(1.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Divide),
            StdVal::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Num(1.0), Operation::Divide),
            StdVal::Num(0.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(false), Operation::Divide),
            StdVal::Num(f64::INFINITY)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Divide),
            StdVal::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("b".to_string())),
                Operation::Divide
            ),
            StdVal::Undefined
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("b".to_string())),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::Divide
            ),
            StdVal::Undefined
        );
    }

    #[test]
    fn test_eval_binary_is_equal() {
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(2.0), Operation::IsEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(1.0), Operation::IsEqual),
            StdVal::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(true), Operation::IsEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(false), Operation::IsEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(true), Operation::IsEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(false), Operation::IsEqual),
            StdVal::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::IsEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::IsEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Num(1.0), Operation::IsEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(false), Operation::IsEqual),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Num(1.0),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::IsEqual
            ),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Num(1.0),
                Operation::IsEqual
            ),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("b".to_string())),
                Operation::IsEqual
            ),
            StdVal::Bool(false)
        );
    }

    #[test]
    fn test_eval_binary_is_not_equal() {
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(2.0), Operation::IsNotEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(1.0), Operation::IsNotEqual),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(true), Operation::IsNotEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(false), Operation::IsNotEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(true), Operation::IsNotEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(false), Operation::IsNotEqual),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::IsNotEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::IsNotEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Num(1.0), Operation::IsNotEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(false), Operation::IsNotEqual),
            StdVal::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Num(1.0),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::IsNotEqual
            ),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Num(1.0),
                Operation::IsNotEqual
            ),
            StdVal::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("b".to_string())),
                Operation::IsNotEqual
            ),
            StdVal::Bool(true)
        );
    }

    #[test]
    fn test_eval_binary_less() {
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(1.0), Operation::Less),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(2.0), Operation::Less),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(2.0), StdVal::Num(1.0), Operation::Less),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(true), Operation::Less),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(false), Operation::Less),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(true), Operation::Less),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(false), Operation::Less),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::Less),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Less),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Num(1.0), Operation::Less),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(false), Operation::Less),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Num(1.0),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::Less
            ),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Num(1.0),
                Operation::Less
            ),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::Less
            ),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("b".to_string())),
                Operation::Less
            ),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("b".to_string())),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::Less
            ),
            StdVal::Bool(false)
        );
    }

    #[test]
    fn test_eval_binary_less_equal() {
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(1.0), Operation::LessEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(2.0), Operation::LessEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(2.0), StdVal::Num(1.0), Operation::LessEqual),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(true), Operation::LessEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(false), Operation::LessEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(true), Operation::LessEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(false), Operation::LessEqual),
            StdVal::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::LessEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::LessEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Num(1.0), Operation::LessEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(false), Operation::LessEqual),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Num(1.0),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::LessEqual
            ),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Num(1.0),
                Operation::LessEqual
            ),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::LessEqual
            ),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("b".to_string())),
                Operation::LessEqual
            ),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("b".to_string())),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::LessEqual
            ),
            StdVal::Bool(false)
        );
    }

    #[test]
    fn test_eval_binary_greater() {
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(1.0), Operation::Greater),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(2.0), Operation::Greater),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(2.0), StdVal::Num(1.0), Operation::Greater),
            StdVal::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(true), Operation::Greater),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(false), Operation::Greater),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(true), Operation::Greater),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(false), Operation::Greater),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::Greater),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Greater),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Num(1.0), Operation::Greater),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(false), Operation::Greater),
            StdVal::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Num(1.0),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::Greater
            ),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Num(1.0),
                Operation::Greater
            ),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::Greater
            ),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("b".to_string())),
                Operation::Greater
            ),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("b".to_string())),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::Greater
            ),
            StdVal::Bool(true)
        );
    }

    #[test]
    fn test_eval_binary_greater_equal() {
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(1.0), Operation::GreaterEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Num(2.0), Operation::GreaterEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(2.0), StdVal::Num(1.0), Operation::GreaterEqual),
            StdVal::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(true), Operation::GreaterEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Bool(false), Operation::GreaterEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(true), Operation::GreaterEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Bool(false), Operation::GreaterEqual),
            StdVal::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::GreaterEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::GreaterEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(false), StdVal::Num(1.0), Operation::GreaterEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(false), Operation::GreaterEqual),
            StdVal::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Num(1.0),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::GreaterEqual
            ),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Num(1.0),
                Operation::GreaterEqual
            ),
            StdVal::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::GreaterEqual
            ),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Str(Arc::new("b".to_string())),
                Operation::GreaterEqual
            ),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("b".to_string())),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::GreaterEqual
            ),
            StdVal::Bool(true)
        );
    }

    #[test]
    fn test_eval_binary_nan() {
        assert_eq!(
            eval_binary(StdVal::Undefined, StdVal::Num(1.0), Operation::Add),
            StdVal::Undefined
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Undefined, Operation::Add),
            StdVal::Undefined
        );
        assert_eq!(
            eval_binary(StdVal::Undefined, StdVal::Num(1.0), Operation::Subtract),
            StdVal::Undefined
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Undefined, Operation::Subtract),
            StdVal::Undefined
        );
        assert_eq!(
            eval_binary(StdVal::Undefined, StdVal::Num(1.0), Operation::Multiply),
            StdVal::Undefined
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Undefined, Operation::Multiply),
            StdVal::Undefined
        );
        assert_eq!(
            eval_binary(StdVal::Undefined, StdVal::Num(1.0), Operation::Divide),
            StdVal::Undefined
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Undefined, Operation::Divide),
            StdVal::Undefined
        );
        assert_eq!(
            eval_binary(StdVal::Undefined, StdVal::Num(1.0), Operation::IsEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Undefined, Operation::IsEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Undefined, StdVal::Num(1.0), Operation::IsNotEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Undefined, Operation::IsNotEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Undefined, StdVal::Num(1.0), Operation::Less),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Undefined, Operation::Less),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Undefined, StdVal::Num(1.0), Operation::LessEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Undefined, Operation::LessEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Undefined, StdVal::Num(1.0), Operation::Greater),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Undefined, Operation::Greater),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Undefined, StdVal::Num(1.0), Operation::GreaterEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Undefined, Operation::GreaterEqual),
            StdVal::Bool(false)
        );
    }

    #[test]
    fn test_eval_binary_coercion() {
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Add),
            StdVal::Num(2.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Subtract),
            StdVal::Num(0.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Multiply),
            StdVal::Num(1.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Divide),
            StdVal::Num(1.0)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::IsEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::IsNotEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Less),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::LessEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::Greater),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Num(1.0), StdVal::Bool(true), Operation::GreaterEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Bool(true),
                Operation::Add
            ),
            StdVal::Str(Arc::new("atrue".to_string()))
        );
        assert_eq!(
            eval_binary(
                StdVal::Str(Arc::new("a".to_string())),
                StdVal::Bool(true),
                Operation::Less
            ),
            StdVal::Bool(false)
        );
        //

        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::Add),
            StdVal::Num(2.0)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::Subtract),
            StdVal::Num(0.0)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::Multiply),
            StdVal::Num(1.0)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::Divide),
            StdVal::Num(1.0)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::IsEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::IsNotEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::Less),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::LessEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::Greater),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(StdVal::Bool(true), StdVal::Num(1.0), Operation::GreaterEqual),
            StdVal::Bool(true)
        );
        assert_eq!(
            eval_binary(
                StdVal::Bool(true),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::Add
            ),
            StdVal::Str(Arc::new("truea".to_string()))
        );
        assert_eq!(
            eval_binary(
                StdVal::Bool(true),
                StdVal::Str(Arc::new("a".to_string())),
                Operation::Less
            ),
            StdVal::Bool(false)
        );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::value::eval::eval_binary;
    use proptest::prelude::*;

    // Strategy for generating RV values
    fn rv_strategy() -> impl Strategy<Value = StdVal> {
        prop_oneof![
            Just(StdVal::Undefined),
            any::<bool>().prop_map(StdVal::Bool),
            any::<f64>()
                .prop_filter("finite numbers", |x| x.is_finite())
                .prop_map(StdVal::Num),
            "[a-zA-Z0-9]*".prop_map(|s| StdVal::Str(Arc::new(s))),
            // For simplicity, we'll skip arrays and objects in basic tests
        ]
    }

    // Strategy for numeric RV values only
    fn numeric_rv_strategy() -> impl Strategy<Value = StdVal> {
        prop_oneof![
            any::<bool>().prop_map(StdVal::Bool),
            any::<f64>()
                .prop_filter("finite numbers", |x| x.is_finite())
                .prop_map(StdVal::Num),
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
            let zero = StdVal::Num(0.0);
            let result = eval_binary(a.clone(), zero, Operation::Add);
            if let Some(num) = a.as_number() {
                prop_assert_eq!(result, StdVal::Num(num));
            }
        }

        // Property: Multiplying by one should be identity
        #[test]
        fn multiplying_by_one_is_identity(a in numeric_rv_strategy()) {
            let one = StdVal::Num(1.0);
            let result = eval_binary(a.clone(), one, Operation::Multiply);
            if let Some(num) = a.as_number() {
                prop_assert_eq!(result, StdVal::Num(num));
            }
        }

        // Property: Equality should be reflexive
        #[test]
        fn equality_is_reflexive(a in rv_strategy()) {
            let result = eval_binary(a.clone(), a, Operation::IsEqual);
            prop_assert_eq!(result, StdVal::Bool(true));
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
            if a_less_b == StdVal::Bool(true) {
                prop_assert_eq!(b_less_a, StdVal::Bool(false));
            }
        }

        // Property: Operations with Undefined should return appropriate results
        #[test]
        fn undefined_operations(
            a in rv_strategy().prop_filter("not undefined", |rv| !matches!(rv, StdVal::Undefined)),
            op in binary_operation_strategy()
        ) {
            let result1 = eval_binary(StdVal::Undefined, a.clone(), op);
            let result2 = eval_binary(a, StdVal::Undefined, op);

            match op {
                Operation::IsEqual | Operation::Is => {
                    prop_assert_eq!(result1, StdVal::Bool(false));
                    prop_assert_eq!(result2, StdVal::Bool(false));
                }
                Operation::IsNotEqual | Operation::IsNot => {
                    prop_assert_eq!(result1, StdVal::Bool(true));
                    prop_assert_eq!(result2, StdVal::Bool(true));
                }
                Operation::Add | Operation::Subtract | Operation::Multiply | Operation::Divide => {
                    prop_assert_eq!(result1, StdVal::Undefined);
                    prop_assert_eq!(result2, StdVal::Undefined);
                }
                Operation::Less | Operation::LessEqual | Operation::Greater | Operation::GreaterEqual => {
                    prop_assert_eq!(result1, StdVal::Bool(false));
                    prop_assert_eq!(result2, StdVal::Bool(false));
                }
                _ => {} // Skip other operations
            }
        }

        // Special case: Undefined compared to itself
        #[test]
        fn undefined_vs_undefined_operations(
            op in binary_operation_strategy()
        ) {
            let result = eval_binary(StdVal::Undefined, StdVal::Undefined, op);

            match op {
                Operation::IsEqual | Operation::Is => {
                    prop_assert_eq!(result, StdVal::Bool(true));
                }
                Operation::IsNotEqual | Operation::IsNot => {
                    prop_assert_eq!(result, StdVal::Bool(false));
                }
                Operation::Add | Operation::Subtract | Operation::Multiply | Operation::Divide => {
                    prop_assert_eq!(result, StdVal::Undefined);
                }
                Operation::Less | Operation::Greater => {
                    prop_assert_eq!(result, StdVal::Bool(false));
                }
                Operation::LessEqual | Operation::GreaterEqual => {
                    // Undefined == Undefined, so LessEqual and GreaterEqual should be true
                    prop_assert_eq!(result, StdVal::Bool(true));
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
            let rv1 = StdVal::Str(Arc::new(s1.clone()));
            let rv2 = StdVal::Str(Arc::new(s2.clone()));
            let result = eval_binary(rv1, rv2, Operation::Add);

            if let StdVal::Str(result_str) = result {
                prop_assert!(result_str.len() >= s1.len());
                prop_assert!(result_str.len() >= s2.len());
                prop_assert_eq!(result_str.len(), s1.len() + s2.len());
            }
        }

        // Property: Division by zero should handle edge cases correctly
        #[test]
        fn division_by_zero_handling(a in numeric_rv_strategy()) {
            let zero = StdVal::Num(0.0);
            let result = eval_binary(a.clone(), zero, Operation::Divide);

            if let Some(num) = a.as_number() {
                if num == 0.0 {
                    // 0/0 should be undefined
                    prop_assert_eq!(result, StdVal::Undefined);
                } else {
                    // Non-zero/0 should be infinity
                    if let StdVal::Num(result_num) = result {
                        prop_assert!(result_num.is_infinite());
                    }
                }
            }
        }

        // Property: Boolean to number conversion should be consistent
        #[test]
        fn boolean_number_conversion_consistency(b in any::<bool>()) {
            let rv_bool = StdVal::Bool(b);
            let expected_num = if b { 1.0 } else { 0.0 };

            prop_assert_eq!(rv_bool.as_number(), Some(expected_num));

            // Test in arithmetic operations
            let result = eval_binary(rv_bool, StdVal::Num(0.0), Operation::Add);
            prop_assert_eq!(result, StdVal::Num(expected_num));
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
            prop_assert!(matches!(result, StdVal::Bool(_)));
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
            let rv_num = StdVal::Num(num);
            let rv_bool = StdVal::Bool(bool_val);
            let expected_bool_as_num = if bool_val { 1.0 } else { 0.0 };

            // num + bool should equal num + bool_as_number
            let result1 = eval_binary(rv_num.clone(), rv_bool.clone(), Operation::Add);
            let result2 = eval_binary(rv_num, StdVal::Num(expected_bool_as_num), Operation::Add);

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
            let rv1 = StdVal::Str(Arc::new(s1.clone()));
            let rv2 = StdVal::Str(Arc::new(s2.clone()));

            let less_result = eval_binary(rv1.clone(), rv2.clone(), Operation::Less);
            let equal_result = eval_binary(rv1, rv2, Operation::IsEqual);

            match s1.cmp(&s2) {
                std::cmp::Ordering::Less => prop_assert_eq!(less_result, StdVal::Bool(true)),
                std::cmp::Ordering::Equal => prop_assert_eq!(equal_result, StdVal::Bool(true)),
                std::cmp::Ordering::Greater => prop_assert_eq!(less_result, StdVal::Bool(false)),
            }
        }

        // Property: Truthiness should be consistent with as_bool
        #[test]
        fn truthiness_consistency(rv in rv_strategy()) {
            let expected_bool = rv.as_bool();

            // Test against known truthy/falsy values
            let false_rv = StdVal::Bool(false);

            if expected_bool {
                // If rv is truthy, it should not equal false
                let ne_false = eval_binary(rv, false_rv, Operation::IsNotEqual);
                prop_assert_eq!(ne_false, StdVal::Bool(true));
            } else {
                // If rv is falsy, specific falsy values should behave consistently
                match rv {
                    StdVal::Num(n) if n == 0.0 => {
                        let eq_false = eval_binary(StdVal::Num(0.0), StdVal::Bool(false), Operation::IsEqual);
                        prop_assert_eq!(eq_false, StdVal::Bool(true));
                    }
                    StdVal::Bool(false) => {
                        let eq_false = eval_binary(StdVal::Bool(false), StdVal::Bool(false), Operation::IsEqual);
                        prop_assert_eq!(eq_false, StdVal::Bool(true));
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
            let rv1 = StdVal::Str(Arc::new(s1.clone()));
            let rv2 = StdVal::Str(Arc::new(s2.clone()));
            let result = eval_binary(rv1, rv2, Operation::Add);

            if let StdVal::Str(result_str) = result {
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
                (-1e-300..1e-300_f64).prop_map(StdVal::Num),
                any::<bool>().prop_map(StdVal::Bool)
            ],
            b in prop_oneof![
                (-1e-300..1e-300_f64).prop_map(StdVal::Num),
                any::<bool>().prop_map(StdVal::Bool)
            ],
            op in prop_oneof![
                Just(Operation::Add),
                Just(Operation::Subtract),
                Just(Operation::Multiply)
            ]
        ) {
            let result = eval_binary(a.clone(), b.clone(), op);

            // Very small numbers should still produce finite results
            if let StdVal::Num(n) = result {
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
            let str_rv = StdVal::Str(Arc::new(s.clone()));
            let num_rv = StdVal::Num(42.0);

            // Test comparison operations
            let eq_result = eval_binary(str_rv.clone(), num_rv.clone(), Operation::IsEqual);
            let lt_result = eval_binary(str_rv.clone(), num_rv.clone(), Operation::Less);

            // These should always produce boolean results
            prop_assert!(matches!(eq_result, StdVal::Bool(_)), "String-number equality should return boolean");
            prop_assert!(matches!(lt_result, StdVal::Bool(_)), "String-number comparison should return boolean");
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
            let rv_num = StdVal::Num(num);
            let rv_bool = StdVal::Bool(bool_val);
            let bool_as_num = StdVal::Num(if bool_val { 1.0 } else { 0.0 });

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
            let result = eval_binary(StdVal::Num(dividend), StdVal::Num(divisor), Operation::Divide);

            if let StdVal::Num(n) = result {
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
            let str_rv = StdVal::Str(Arc::new(s.clone()));
            let num_rv = StdVal::Num(num);

            let result1 = eval_binary(str_rv.clone(), num_rv.clone(), Operation::Add);
            let result2 = eval_binary(num_rv, str_rv, Operation::Add);

            if let (StdVal::Str(s1), StdVal::Str(s2)) = (result1, result2) {
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
            if a_lt_b == StdVal::Bool(true) && b_lt_c == StdVal::Bool(true) {
                prop_assert_eq!(a_lt_c, StdVal::Bool(true), "Less-than should be transitive");
            }
        }

        // Property: Subtraction is inverse of addition (within floating-point precision limits)
        #[test]
        fn subtraction_inverse_of_addition_small_numbers(
            a in (-1e6..1e6_f64).prop_map(StdVal::Num),
            b in (-1e6..1e6_f64).prop_map(StdVal::Num)
        ) {
            // (a + b) - b should equal a
            let sum = eval_binary(a.clone(), b.clone(), Operation::Add);
            let difference = eval_binary(sum, b, Operation::Subtract);

            if let (StdVal::Num(original), StdVal::Num(result)) = (a, difference) {
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
            let zero = StdVal::Num(0.0);
            let result1 = eval_binary(a.clone(), zero.clone(), Operation::Multiply);
            let result2 = eval_binary(zero, a, Operation::Multiply);

            prop_assert_eq!(result1, StdVal::Num(0.0), "Multiplication by zero should yield zero");
            prop_assert_eq!(result2, StdVal::Num(0.0), "Multiplication by zero should yield zero");
        }

        // Property: String comparison with empty strings
        #[test]
        fn string_comparison_with_empty_strings(
            s in "[a-zA-Z0-9]*"
        ) {
            let empty_str = StdVal::Str(Arc::new("".to_string()));
            let non_empty_str = StdVal::Str(Arc::new(s.clone()));

            let empty_lt_nonempty = eval_binary(empty_str.clone(), non_empty_str.clone(), Operation::Less);
            let nonempty_gt_empty = eval_binary(non_empty_str, empty_str, Operation::Greater);

            if !s.is_empty() {
                prop_assert_eq!(empty_lt_nonempty, StdVal::Bool(true), "Empty string should be less than non-empty");
                prop_assert_eq!(nonempty_gt_empty, StdVal::Bool(true), "Non-empty string should be greater than empty");
            }
        }
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    use crate::value::eval::eval_binary;

    /// These tests capture specific edge cases found through property testing
    /// or known to be important for the domain
    #[test]
    fn regression_nan_handling() {
        // NaN comparisons should always be false
        let nan = StdVal::Num(f64::NAN);
        let num = StdVal::Num(1.0);

        assert_eq!(
            eval_binary(nan.clone(), num.clone(), Operation::Less),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(nan.clone(), num.clone(), Operation::Greater),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(nan.clone(), num.clone(), Operation::IsEqual),
            StdVal::Bool(false)
        );
        assert_eq!(
            eval_binary(nan.clone(), nan.clone(), Operation::IsEqual),
            StdVal::Bool(false)
        );
    }

    #[test]
    fn regression_infinity_arithmetic() {
        let inf = StdVal::Num(f64::INFINITY);
        let num = StdVal::Num(42.0);

        // inf + num = inf
        assert_eq!(eval_binary(inf.clone(), num.clone(), Operation::Add), inf);
        // inf - inf should be NaN, but we might handle it differently
        let result = eval_binary(inf.clone(), inf.clone(), Operation::Subtract);
        match result {
            StdVal::Num(n) => assert!(n.is_nan()),
            StdVal::Undefined => {} // Also acceptable
            _ => panic!("Unexpected result for inf - inf"),
        }
    }

    #[test]
    fn regression_string_number_coercion() {
        // String that can be parsed as number
        let str_num = StdVal::Str(Arc::new("42".to_string()));
        let num = StdVal::Num(42.0);

        // In comparisons, string numbers should be coerced
        let result = eval_binary(str_num, num, Operation::IsEqual);
        // This depends on implementation - document the expected behavior
        match result {
            StdVal::Bool(_) => {} // Either true or false is acceptable, but should be documented
            _ => panic!("String-number comparison should return boolean"),
        }
    }

    #[test]
    fn regression_addition_associativity_large_numbers() {
        // This test documents a known floating-point precision issue
        // Found through property testing: very large numbers don't maintain exact associativity
        // Failing case from proptest: a = 3.471971265526436e290, b = 1.1414671723104438e295, c = -1.563421480094111e295
        let a = StdVal::Num(3.471971265526436e290);
        let b = StdVal::Num(1.1414671723104438e295);
        let c = StdVal::Num(-1.563421480094111e295);

        // (a + b) + c
        let ab = eval_binary(a.clone(), b.clone(), Operation::Add);
        let ab_c = eval_binary(ab, c.clone(), Operation::Add);

        // a + (b + c)
        let bc = eval_binary(b, c, Operation::Add);
        let a_bc = eval_binary(a, bc, Operation::Add);

        // These should be approximately equal but may not be exactly equal due to floating-point precision
        match (ab_c, a_bc) {
            (StdVal::Num(left), StdVal::Num(right)) => {
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
        let a = StdVal::Num(239305169.8616219);
        let b = StdVal::Num(-4330652516.771584);

        // (a + b) - b should equal a
        let sum = eval_binary(a.clone(), b.clone(), Operation::Add);
        let difference = eval_binary(sum, b, Operation::Subtract);

        match (a, difference) {
            (StdVal::Num(original), StdVal::Num(result)) => {
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
        let a = StdVal::Num(776327.478839819);
        let b = StdVal::Num(333251.5209597033);

        // (a + b) - b should equal a
        let sum = eval_binary(a.clone(), b.clone(), Operation::Add);
        let difference = eval_binary(sum, b, Operation::Subtract);

        match (a, difference) {
            (StdVal::Num(original), StdVal::Num(result)) => {
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
        let a = StdVal::Num(-8.856050288210175e-102);
        let b = StdVal::Num(5.003750612076637e-88);
        let c = StdVal::Num(-5.226269442637776e-88);

        // (a + b) + c vs a + (b + c)
        let ab = eval_binary(a.clone(), b.clone(), Operation::Add);
        let ab_c = eval_binary(ab, c.clone(), Operation::Add);

        let bc = eval_binary(b, c, Operation::Add);
        let a_bc = eval_binary(a, bc, Operation::Add);

        match (ab_c, a_bc) {
            (StdVal::Num(left), StdVal::Num(right)) => {
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
        let a = StdVal::Num(-13542.264609919892);
        let b = StdVal::Num(751876.9370472291);

        let sum = eval_binary(a.clone(), b.clone(), Operation::Add);
        let difference = eval_binary(sum, b, Operation::Subtract);

        match (a, difference) {
            (StdVal::Num(original), StdVal::Num(result)) => {
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
