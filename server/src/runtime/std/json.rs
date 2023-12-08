use serde_json::json;
use std::rc::Rc;

use crate::runtime::interpreter::{HaltReason, InterpretError, Interpreter};
use crate::runtime::types::RV;

pub fn nt_json_encode(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    Ok(RV::Str(Rc::new(json!(args[0]).to_string())))
}

pub fn nt_json_decode(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    let json_str = match &args[0] {
        RV::Str(s) => s,
        _ => {
            return Err(HaltReason::Error(InterpretError::Other {
                message: format!("json_decode: Unexpected argument '{:?}'", args[0]),
            }))
        }
    };

    let parsed: RV = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            return Err(HaltReason::Error(InterpretError::Other {
                message: format!("json_decode: Unhandled error '{:?}'", e),
            }))
        }
    };

    Ok(parsed)
}
