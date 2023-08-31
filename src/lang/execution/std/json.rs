use std::rc::Rc;

use serde_json::json;

use crate::lang::execution::{primitives::{HaltReason, RV}, interpreter::Interpreter};

pub fn nt_json_encode(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    return Ok(RV::Str(Rc::new(json!(args[0]).to_string())));
}

pub fn nt_json_decode(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    let json_str = match &args[0] {
        RV::Str(s) => s,
        _ => return Err(HaltReason::Error("json_decode: expected string".to_string()))
    };

    let parsed: RV = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => return Err(HaltReason::Error(format!("json_decode: {}", e)))
    };

    Ok(parsed)
}