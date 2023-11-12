use crate::runtime::interpreter::{HaltReason, Interpreter};
use crate::runtime::types::RV;

fn _calculate(n: f64) -> f64 {
    if n < 2. {
        return n;
    }
    _calculate(n - 1.) + _calculate(n - 2.)
}

pub fn nt_fib(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    if let RV::Num(n) = args[0] {
        return Ok(RV::Num(_calculate(n)));
    }
    Err(HaltReason::GenericError(
        "Unexpected types for bench function".to_owned(),
    ))
}
