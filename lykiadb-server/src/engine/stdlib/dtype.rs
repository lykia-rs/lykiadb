use rustc_hash::FxHashMap;

use crate::{
    engine::interpreter::{HaltReason, InterpretError, Interpreter},
    util::Shared,
    value::{datatype::Datatype, RV},
};

pub fn nt_of(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    Ok(RV::Datatype(args[0].get_type()))
}

pub fn nt_array_of(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    match &args[0] {
        RV::Datatype(inner) => Ok(RV::Datatype(Datatype::Array(Box::new(inner.clone())))),
        _ => Err(HaltReason::Error(
            InterpretError::Other {
                message: format!("array_of: Unexpected argument '{:?}'", args[0]),
            }
            .into(),
        )),
    }
}

fn object_rec(inner: Shared<FxHashMap<String, RV>>) -> Result<Datatype, HaltReason> {
    let mut type_map: FxHashMap<String, Datatype> = FxHashMap::default();
    for (key, value) in inner.read().unwrap().iter() {
        match value {
            RV::Object(inner) => {
                let inner = inner.clone();
                type_map.insert(key.clone(), object_rec(inner)?);
            }
            RV::Datatype(rvdt) => {
                type_map.insert(key.clone(), rvdt.clone());
            }
            _ => {}
        }
    }
    Ok(Datatype::Object(type_map))
}

pub fn nt_object_of(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    match &args[0] {
        RV::Object(inner) => Ok(RV::Datatype(object_rec(inner.clone())?)),
        _ => Err(HaltReason::Error(
            InterpretError::Other {
                message: format!("object_of: Unexpected argument '{:?}'", args[0]),
            }
            .into(),
        )),
    }
}
