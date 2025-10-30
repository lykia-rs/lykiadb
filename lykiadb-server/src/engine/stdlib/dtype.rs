use lykiadb_lang::types::Datatype;
use rustc_hash::FxHashMap;

use crate::{
    engine::interpreter::{HaltReason, InterpretError, Interpreter},
    value::{Value, ValueObject},
};

pub fn nt_of<V: Value>(_interpreter: &mut Interpreter<V>, args: &[V]) -> Result<V, HaltReason<V>> {
    Ok(V::datatype(args[0].get_type()))
}

pub fn nt_array_of<V: Value>(_interpreter: &mut Interpreter<V>, args: &[V]) -> Result<V, HaltReason<V>> {
    if let Some(inner) = args[0].as_datatype() {
        Ok(V::datatype(Datatype::Array(Box::new(inner.clone()))))
    } else {
        Err(HaltReason::Error(
            InterpretError::Other {
                message: format!("array_of: Unexpected argument '{:?}'", args[0]),
            }
            .into(),
        ))
    }
}

pub fn nt_callable_of<V: Value>(_interpreter: &mut Interpreter<V>, args: &[V]) -> Result<V, HaltReason<V>> {
    if let (Some(input), Some(output)) = (args[0].as_datatype(), args[1].as_datatype()) {
        Ok(V::datatype(Datatype::Callable(
            Box::new(input.clone()),
            Box::new(output.clone()),
        )))
    } else {
        Err(HaltReason::Error(
            InterpretError::Other {
                message: format!("callable_of: Unexpected arguments '{:?}', '{:?}'", args[0], args[1]),
            }
            .into(),
        ))
    }
}

pub fn nt_tuple_of<V: Value>(_interpreter: &mut Interpreter<V>, args: &[V]) -> Result<V, HaltReason<V>> {
    let mut inner = Vec::new();
    for arg in args {
        if let Some(dt) = arg.as_datatype() {
            inner.push(dt.clone());
        } else {
            return Err(HaltReason::Error(
                InterpretError::Other {
                    message: format!("tuple_of: Unexpected argument '{arg:?}'"),
                }
                .into(),
            ));
        }
    }
    Ok(V::datatype(Datatype::Tuple(inner)))
}

fn object_rec<V: Value>(inner: &V::Object) -> Result<Datatype, HaltReason<V>> {
    let mut type_map: FxHashMap<String, Datatype> = FxHashMap::default();
    for (key, value) in inner.iter() {
        if let Some(object_inner) = value.as_object() {
            type_map.insert(key.clone(), object_rec(&object_inner)?);
        } else if let Some(rvdt) = value.as_datatype() {
            type_map.insert(key.clone(), rvdt.clone());
        }
    }
    Ok(Datatype::Object(type_map))
}

pub fn nt_object_of<V: Value>(_interpreter: &mut Interpreter<V>, args: &[V]) -> Result<V, HaltReason<V>> {
    if let Some(inner) = args[0].as_object() {
        Ok(V::datatype(object_rec(&inner)?))
    } else {
        Err(HaltReason::Error(
            InterpretError::Other {
                message: format!("object_of: Unexpected argument '{:?}'", args[0]),
            }
            .into(),
        ))
    }
}
