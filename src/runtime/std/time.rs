use std::time;
use crate::runtime::interpreter::Interpreter;
use crate::lang::types::{RV, CallableError};

pub fn nt_clock(_interpreter: &mut Interpreter, _args: &[RV]) -> Result<RV, CallableError> {
    if let Ok(n) = time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        return Ok(RV::Num(n.as_secs_f64()));
    }
    Ok(RV::Undefined)
}