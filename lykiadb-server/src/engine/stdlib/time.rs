use crate::engine::interpreter::{HaltReason, Interpreter};
use crate::value::RV;
use std::time;

pub fn nt_clock(_interpreter: &mut Interpreter, _args: &[RV]) -> Result<RV, HaltReason> {
    if let Ok(n) = time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        return Ok(RV::Num(n.as_secs_f64()));
    }
    Ok(RV::Undefined)
}
