use std::rc::Rc;

use crate::lang::token::Symbol::*;
use crate::lang::token::Token;
use crate::lang::token::TokenType::Symbol;
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
pub fn coerce2number(val: RV) -> RV {
    match val {
        RV::Num(value) => RV::Num(-value),
        RV::Bool(true) => RV::Num(-1.0),
        RV::Bool(false) => RV::Num(0.0),
        _ => RV::NaN,
    }
}

#[inline(always)]
pub fn eval_binary(left_eval: RV, right_eval: RV, tok: &Token) -> RV {
    let tok_type = tok.tok_type.clone();

    let (left_coerced, right_coerced) = match (&left_eval, &tok_type, &right_eval) {
        (RV::Num(n), _, RV::Bool(bool)) => (RV::Num(*n), RV::Num(bool2num!(*bool))),
        (RV::Bool(bool), _, RV::Num(n)) => (RV::Num(bool2num!(*bool)), RV::Num(*n)),
        (RV::Bool(l), Symbol(Plus), RV::Bool(r))
        | (RV::Bool(l), Symbol(Minus), RV::Bool(r))
        | (RV::Bool(l), Symbol(Star), RV::Bool(r))
        | (RV::Bool(l), Symbol(Slash), RV::Bool(r)) => {
            (RV::Num(bool2num!(*l)), RV::Num(bool2num!(*r)))
        }
        (_, _, _) => (left_eval, right_eval),
    };

    match (left_coerced, tok_type, right_coerced) {
        (RV::Null, Symbol(EqualEqual), RV::Null) => RV::Bool(true),
        (RV::Null, Symbol(BangEqual), RV::Null) => RV::Bool(false),
        //
        (_, Symbol(EqualEqual), RV::Null) | (RV::Null, Symbol(EqualEqual), _) => RV::Bool(false),
        //
        (RV::NaN, Symbol(Plus), _)
        | (_, Symbol(Plus), RV::NaN)
        | (RV::NaN, Symbol(Minus), _)
        | (_, Symbol(Minus), RV::NaN)
        | (RV::NaN, Symbol(Star), _)
        | (_, Symbol(Star), RV::NaN)
        | (RV::NaN, Symbol(Slash), _)
        | (_, Symbol(Slash), RV::NaN) => RV::NaN,
        //
        (RV::Num(l), Symbol(Plus), RV::Num(r)) => RV::Num(l + r),
        (RV::Num(l), Symbol(Minus), RV::Num(r)) => RV::Num(l - r),
        (RV::Num(l), Symbol(Star), RV::Num(r)) => RV::Num(l * r),
        (RV::Num(l), Symbol(Slash), RV::Num(r)) => RV::Num(l / r),
        (RV::Num(l), Symbol(Less), RV::Num(r)) => RV::Bool(l < r),
        (RV::Num(l), Symbol(LessEqual), RV::Num(r)) => RV::Bool(l <= r),
        (RV::Num(l), Symbol(Greater), RV::Num(r)) => RV::Bool(l > r),
        (RV::Num(l), Symbol(GreaterEqual), RV::Num(r)) => RV::Bool(l >= r),
        (RV::Num(l), Symbol(BangEqual), RV::Num(r)) => RV::Bool(l != r),
        (RV::Num(l), Symbol(EqualEqual), RV::Num(r)) => RV::Bool(l == r),
        //
        (RV::Str(l), Symbol(Plus), RV::Str(r)) => RV::Str(Rc::new(l.to_string() + &r.to_string())),
        (RV::Str(l), Symbol(Less), RV::Str(r)) => RV::Bool(l < r),
        (RV::Str(l), Symbol(LessEqual), RV::Str(r)) => RV::Bool(l <= r),
        (RV::Str(l), Symbol(Greater), RV::Str(r)) => RV::Bool(l > r),
        (RV::Str(l), Symbol(GreaterEqual), RV::Str(r)) => RV::Bool(l >= r),
        (RV::Str(l), Symbol(BangEqual), RV::Str(r)) => RV::Bool(l != r),
        (RV::Str(l), Symbol(EqualEqual), RV::Str(r)) => RV::Bool(l == r),
        //
        (RV::Bool(l), Symbol(Less), RV::Bool(r)) => RV::Bool(!l & r),
        (RV::Bool(l), Symbol(LessEqual), RV::Bool(r)) => RV::Bool(l <= r),
        (RV::Bool(l), Symbol(Greater), RV::Bool(r)) => RV::Bool(l & !r),
        (RV::Bool(l), Symbol(GreaterEqual), RV::Bool(r)) => RV::Bool(l >= r),
        (RV::Bool(l), Symbol(BangEqual), RV::Bool(r)) => RV::Bool(l != r),
        (RV::Bool(l), Symbol(EqualEqual), RV::Bool(r)) => RV::Bool(l == r),
        //
        (RV::Str(s), Symbol(Plus), RV::Num(num)) => {
            RV::Str(Rc::new(s.to_string() + &num.to_string()))
        }
        (RV::Num(num), Symbol(Plus), RV::Str(s)) => {
            RV::Str(Rc::new(num.to_string() + &s.to_string()))
        }
        //
        (RV::Str(s), Symbol(Plus), RV::Bool(bool)) => {
            RV::Str(Rc::new(s.to_string() + &bool.to_string()))
        }
        (RV::Bool(bool), Symbol(Plus), RV::Str(s)) => {
            RV::Str(Rc::new(bool.to_string() + &s.to_string()))
        }
        //
        (_, Symbol(Less), _)
        | (_, Symbol(LessEqual), _)
        | (_, Symbol(Greater), _)
        | (_, Symbol(GreaterEqual), _)
        | (_, Symbol(EqualEqual), _)
        | (_, Symbol(BangEqual), _) => RV::Bool(false),
        //
        (_, Symbol(Plus), _)
        | (_, Symbol(Minus), _)
        | (_, Symbol(Star), _)
        | (_, Symbol(Slash), _) => RV::NaN,
        //
        (_, _, _) => RV::Undefined,
    }
}
