use crate::{
    engine::interpreter::{HaltReason, InterpretError, Interpreter},
    value::{datatype::Datatype, RV},
};

pub fn nt_of(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    Ok(RV::Datatype(args[0].get_type()))
}

pub fn nt_array_of(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    match &args[0] {
        RV::Datatype(inner) => Ok(RV::Datatype(Datatype::Array(Box::new(inner.clone())))),
        _ => Err(HaltReason::Error(InterpretError::Other {
            message: format!("array_of: Unexpected argument '{:?}'", args[0]),
        }
        .into())),
    }
}
