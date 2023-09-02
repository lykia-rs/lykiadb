use crate::lang::execution::interpreter::Interpreter;
use crate::lang::parsing::types::{RV, CallableError};

fn _calculate(n: f64) -> f64 {
    if n < 2. {
        return n;
    }
    _calculate(n - 1.) + _calculate(n - 2.)
}

pub fn nt_fib(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, CallableError> {
    if let RV::Num(n) = args[0] {
        return Ok(RV::Num(_calculate(n)));
    }
    Err(CallableError::GenericError("Unexpected types for bench function".to_owned()))
}