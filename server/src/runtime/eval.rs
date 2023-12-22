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
        (RV::Null, Operation::Equal, RV::Null) => RV::Bool(true),
        (RV::Null, Operation::NotEqual, RV::Null) => RV::Bool(false),
        //
        (_, Operation::Equal, RV::Null) | (RV::Null, Operation::Equal, _) => RV::Bool(false),
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
        (RV::Num(l), Operation::Divide, RV::Num(r)) => RV::Num(l / r),
        (RV::Num(l), Operation::Less, RV::Num(r)) => RV::Bool(l < r),
        (RV::Num(l), Operation::LessEqual, RV::Num(r)) => RV::Bool(l <= r),
        (RV::Num(l), Operation::Greater, RV::Num(r)) => RV::Bool(l > r),
        (RV::Num(l), Operation::GreaterEqual, RV::Num(r)) => RV::Bool(l >= r),
        (RV::Num(l), Operation::NotEqual, RV::Num(r)) => RV::Bool(l != r),
        (RV::Num(l), Operation::Equal, RV::Num(r)) => RV::Bool(l == r),
        //
        (RV::Str(l), Operation::Add, RV::Str(r)) => {
            RV::Str(Rc::new(l.to_string() + &r.to_string()))
        }
        (RV::Str(l), Operation::Less, RV::Str(r)) => RV::Bool(l < r),
        (RV::Str(l), Operation::LessEqual, RV::Str(r)) => RV::Bool(l <= r),
        (RV::Str(l), Operation::Greater, RV::Str(r)) => RV::Bool(l > r),
        (RV::Str(l), Operation::GreaterEqual, RV::Str(r)) => RV::Bool(l >= r),
        (RV::Str(l), Operation::NotEqual, RV::Str(r)) => RV::Bool(l != r),
        (RV::Str(l), Operation::Equal, RV::Str(r)) => RV::Bool(l == r),
        //
        (RV::Bool(l), Operation::Less, RV::Bool(r)) => RV::Bool(!l & r),
        (RV::Bool(l), Operation::LessEqual, RV::Bool(r)) => RV::Bool(l <= r),
        (RV::Bool(l), Operation::Greater, RV::Bool(r)) => RV::Bool(l & !r),
        (RV::Bool(l), Operation::GreaterEqual, RV::Bool(r)) => RV::Bool(l >= r),
        (RV::Bool(l), Operation::NotEqual, RV::Bool(r)) => RV::Bool(l != r),
        (RV::Bool(l), Operation::Equal, RV::Bool(r)) => RV::Bool(l == r),
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
        | (_, Operation::Equal, _)
        | (_, Operation::NotEqual, _) => RV::Bool(false),
        //
        (_, Operation::Add, _)
        | (_, Operation::Subtract, _)
        | (_, Operation::Multiply, _)
        | (_, Operation::Divide, _) => RV::NaN,
        //
        (_, _, _) => RV::Undefined,
    }
}
