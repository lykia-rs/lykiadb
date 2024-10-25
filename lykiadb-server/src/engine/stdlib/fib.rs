use crate::{
    engine::interpreter::{HaltReason, InterpretError, Interpreter},
    value::RV,
};

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
    Err(HaltReason::Error(
        InterpretError::Other {
            message: format!("fib_nat: Unexpected argument '{:?}'", args[0]),
        }
        .into(),
    ))
}
