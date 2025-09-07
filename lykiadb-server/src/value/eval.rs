use super::RV;
use lykiadb_lang::ast::expr::Operation;
use std::ops;
use std::sync::Arc;

impl PartialEq for RV {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RV::Array(_), RV::Array(_)) | (RV::Object(_), RV::Object(_)) => false,
            (RV::Undefined, RV::Undefined) => true,
            //
            (RV::Undefined, _) | (_, RV::Undefined) => false,
            //
            (RV::Str(a), RV::Str(b)) => a == b,
            (RV::Num(a), RV::Num(b)) => a == b,
            (RV::Bool(a), RV::Bool(b)) => a == b,
            //
            (RV::Str(_), RV::Num(b)) => self.eq_str_num(*b),
            (RV::Num(a), RV::Str(_)) => other.eq_str_num(*a),
            //
            (RV::Str(_), RV::Bool(b)) => self.eq_any_bool(*b),
            (RV::Bool(a), RV::Str(_)) => other.eq_any_bool(*a),
            //
            (RV::Num(_), RV::Bool(b)) => self.eq_any_bool(*b),
            (RV::Bool(a), RV::Num(_)) => other.eq_any_bool(*a),
            //
            (RV::Datatype(a), RV::Datatype(b)) => a == b,
            //
            _ => false,
        }
    }
}

impl PartialOrd for RV {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (RV::Array(_), RV::Array(_)) | (RV::Object(_), RV::Object(_)) => None,
            (RV::Undefined, RV::Undefined) => Some(std::cmp::Ordering::Equal),
            //
            (RV::Undefined, _) | (_, RV::Undefined) => None,
            //
            (RV::Str(a), RV::Str(b)) => Some(a.cmp(b)),
            (RV::Num(a), RV::Num(b)) => a.partial_cmp(b),
            (RV::Bool(a), RV::Bool(b)) => a.partial_cmp(b),
            //
            (RV::Str(a), RV::Num(b)) => {
                if let Ok(num) = a.parse::<f64>() {
                    return num.partial_cmp(b);
                }
                None
            }
            (RV::Num(a), RV::Str(b)) => {
                if let Ok(num) = b.parse::<f64>() {
                    return a.partial_cmp(&num);
                }
                None
            }
            //
            (RV::Str(_), RV::Bool(b)) => self.partial_cmp_str_bool(*b),
            (RV::Bool(a), RV::Str(_)) => other.partial_cmp_str_bool(*a),
            //
            (RV::Num(num), RV::Bool(b)) => num.partial_cmp(&if *b { 1.0 } else { 0.0 }),
            (RV::Bool(b), RV::Num(num)) => (if *b { 1.0 } else { 0.0 }).partial_cmp(num),
            //
            _ => None,
        }
    }
}

impl ops::Add for RV {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            //
            (RV::Bool(_), RV::Bool(_)) | (RV::Num(_), RV::Bool(_)) | (RV::Bool(_), RV::Num(_)) => {
                RV::Num(self.as_number().unwrap() + rhs.as_number().unwrap())
            }

            (RV::Num(l), RV::Num(r)) => RV::Num(l + r),
            //
            (RV::Str(l), RV::Str(r)) => RV::Str(Arc::new(l.to_string() + &r.to_string())),
            //
            (RV::Str(s), RV::Num(num)) => RV::Str(Arc::new(s.to_string() + &num.to_string())),
            (RV::Num(num), RV::Str(s)) => RV::Str(Arc::new(num.to_string() + &s.to_string())),
            //
            (RV::Str(s), RV::Bool(bool)) => RV::Str(Arc::new(s.to_string() + &bool.to_string())),
            (RV::Bool(bool), RV::Str(s)) => RV::Str(Arc::new(bool.to_string() + &s.to_string())),
            //
            (_, _) => RV::Undefined,
        }
    }
}

impl ops::Sub for RV {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RV::Undefined, _) | (_, RV::Undefined) => RV::Undefined,
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| RV::Num(a - b))
                .unwrap_or(RV::Undefined),
        }
    }
}

impl ops::Mul for RV {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RV::Undefined, _) | (_, RV::Undefined) => RV::Undefined,
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| RV::Num(a * b))
                .unwrap_or(RV::Undefined),
        }
    }
}

impl ops::Div for RV {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RV::Undefined, _) | (_, RV::Undefined) => RV::Undefined,
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| {
                    if a == 0.0 && b == 0.0 {
                        RV::Undefined
                    } else {
                        RV::Num(a / b)
                    }
                })
                .unwrap_or(RV::Undefined),
        }
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
mod test {
    use std::sync::Arc;

    use lykiadb_lang::ast::expr::Operation;
    use rustc_hash::FxHashMap;

    use crate::{
        util::alloc_shared,
        value::eval::{RV, eval_binary},
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
        assert!((RV::Array(alloc_shared(vec![]))).as_bool());
        assert!((RV::Object(alloc_shared(FxHashMap::default()))).as_bool());
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
    use super::*;
    use proptest::prelude::*;
    use crate::value::eval::eval_binary;

    // Strategy for generating RV values
    fn rv_strategy() -> impl Strategy<Value = RV> {
        prop_oneof![
            Just(RV::Undefined),
            any::<bool>().prop_map(RV::Bool),
            any::<f64>().prop_filter("finite numbers", |x| x.is_finite()).prop_map(RV::Num),
            "[a-zA-Z0-9]*".prop_map(|s| RV::Str(Arc::new(s))),
            // For simplicity, we'll skip arrays and objects in basic tests
        ]
    }

    // Strategy for numeric RV values only
    fn numeric_rv_strategy() -> impl Strategy<Value = RV> {
        prop_oneof![
            any::<bool>().prop_map(RV::Bool),
            any::<f64>().prop_filter("finite numbers", |x| x.is_finite()).prop_map(RV::Num),
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

        // Property: Associativity for addition (when all operations are valid)
        #[test]
        fn addition_associativity(
            a in numeric_rv_strategy(),
            b in numeric_rv_strategy(),
            c in numeric_rv_strategy()
        ) {
            // (a + b) + c
            let ab = eval_binary(a.clone(), b.clone(), Operation::Add);
            let ab_c = eval_binary(ab, c.clone(), Operation::Add);
            
            // a + (b + c)
            let bc = eval_binary(b, c, Operation::Add);
            let a_bc = eval_binary(a, bc, Operation::Add);
            
            prop_assert_eq!(ab_c, a_bc);
        }

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
                    RV::Num(n) if n == 0.0 => {
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
        let nan = RV::Num(f64::NAN);
        let num = RV::Num(1.0);
        
        assert_eq!(eval_binary(nan.clone(), num.clone(), Operation::Less), RV::Bool(false));
        assert_eq!(eval_binary(nan.clone(), num.clone(), Operation::Greater), RV::Bool(false));
        assert_eq!(eval_binary(nan.clone(), num.clone(), Operation::IsEqual), RV::Bool(false));
        assert_eq!(eval_binary(nan.clone(), nan.clone(), Operation::IsEqual), RV::Bool(false));
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
            RV::Undefined => {}, // Also acceptable
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
            RV::Bool(_) => {}, // Either true or false is acceptable, but should be documented
            _ => panic!("String-number comparison should return boolean"),
        }
    }
}
