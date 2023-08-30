use std::time;
use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{Callable, HaltReason, RV};

pub fn nt_clock(_interpreter: &mut Interpreter, _args: &[RV]) -> Result<RV, HaltReason> {
    if let Ok(n) = time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        return Ok(RV::Num(n.as_secs_f64()));
    }
    Ok(RV::Undefined)
}

pub fn nt_bench(_interpreter: &mut Interpreter, _args: &[RV]) -> Result<RV, HaltReason> {
    let benched = &_args[0];
    let repeats = &_args[1];

    if let RV::Callable(_, benched_unwrapped) = benched {
        if let RV::Num(repeat_unwrapped) = repeats {
            let repeat_int = *repeat_unwrapped as i32;
            let mut total: f64 = 0f64;

            for _ in 0..repeat_int {
                let start =  time::SystemTime::now().duration_since(time::UNIX_EPOCH);
                benched_unwrapped.call(_interpreter, Vec::new().as_slice())?;
                let end =  time::SystemTime::now().duration_since(time::UNIX_EPOCH);
                total += end.unwrap().as_secs_f64() - start.unwrap().as_secs_f64();
            }
            return Ok(RV::Num(total / repeat_int as f64));
        }
    }
    Err(HaltReason::Error("Unexpected types for bench function".to_owned()))
}