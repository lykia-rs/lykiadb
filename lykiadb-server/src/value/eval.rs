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
        value::eval::{eval_binary, RV},
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
        assert_eq!(eval_binary(RV::Undefined, RV::Num(1.0), Operation::Add), RV::Undefined);
        assert_eq!(eval_binary(RV::Num(1.0), RV::Undefined, Operation::Add), RV::Undefined);
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
