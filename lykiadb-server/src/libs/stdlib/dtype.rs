use lykiadb_lang::ast::Span;
use rustc_hash::FxHashMap;

use crate::{
    interpreter::interpreter::{HaltReason, InterpretError, Interpreter},
    lykia_module, lykia_native_fn,
    value::{RV, object::RVObject},
};

pub fn nt_of<'rv>(
    _interpreter: &mut Interpreter<'rv>,
    called_from: &Span,
    args: &[RV<'rv>],
) -> Result<RV<'rv>, HaltReason<'rv>> {
    Ok(RV::Datatype(args[0].get_type()))
}

pub fn nt_array_of<'rv>(
    _interpreter: &mut Interpreter<'rv>,
    called_from: &Span,
    args: &[RV<'rv>],
) -> Result<RV<'rv>, HaltReason<'rv>> {
    match &args[0] {
        RV::Datatype(inner) => Ok(RV::Datatype(Datatype::Array(Box::new(inner.clone())))),
        _ => Err(HaltReason::Error(
            InterpretError::InvalidArgumentType {
                span: *called_from,
                expected: "datatype".to_string(),
            }
            .into(),
        )),
    }
}

pub fn nt_callable_of<'rv>(
    _interpreter: &mut Interpreter<'rv>,
    called_from: &Span,
    args: &[RV<'rv>],
) -> Result<RV<'rv>, HaltReason<'rv>> {
    match &args[0] {
        RV::Datatype(input) => match &args[1] {
            RV::Datatype(output) => Ok(RV::Datatype(Datatype::Callable(
                Box::new(input.clone()),
                Box::new(output.clone()),
            ))),
            _ => Err(HaltReason::Error(
                InterpretError::InvalidArgumentType {
                    span: *called_from,
                    expected: "datatype".to_string(),
                }
                .into(),
            )),
        },
        _ => Err(HaltReason::Error(
            InterpretError::InvalidArgumentType {
                span: *called_from,
                expected: "datatype".to_string(),
            }
            .into(),
        )),
    }
}

pub fn nt_tuple_of<'rv>(
    _interpreter: &mut Interpreter<'rv>,
    called_from: &Span,
    args: &[RV<'rv>],
) -> Result<RV<'rv>, HaltReason<'rv>> {
    let mut inner = Vec::new();
    for arg in args {
        match arg {
            RV::Datatype(dt) => inner.push(dt.clone()),
            _ => {
                return Err(HaltReason::Error(
                    InterpretError::InvalidArgumentType {
                        span: *called_from,
                        expected: "datatype".to_string(),
                    }
                    .into(),
                ));
            }
        }
    }
    Ok(RV::Datatype(Datatype::Tuple(inner)))
}

fn object_rec<'rv>(inner: &RVObject<'rv>) -> Result<Datatype, HaltReason<'rv>> {
    let mut type_map: FxHashMap<String, Datatype> = FxHashMap::default();
    for (key, value) in inner.iter() {
        match value {
            RV::Object(inner) => {
                type_map.insert(key.clone(), object_rec(&inner)?);
            }
            RV::Datatype(rvdt) => {
                type_map.insert(key.clone(), rvdt.clone());
            }
            _ => {}
        }
    }
    Ok(Datatype::Object(type_map))
}

pub fn nt_object_of<'rv>(
    _interpreter: &mut Interpreter<'rv>,
    called_from: &Span,
    args: &[RV<'rv>],
) -> Result<RV<'rv>, HaltReason<'rv>> {
    match &args[0] {
        RV::Object(inner) => Ok(RV::Datatype(object_rec(inner)?)),
        _ => Err(HaltReason::Error(
            InterpretError::InvalidArgumentType {
                span: *called_from,
                expected: "datatype".to_string(),
            }
            .into(),
        )),
    }
}

lykia_module!(dtype, {
    of_ => lykia_native_fn!(nt_of),
    array => lykia_native_fn!(nt_array_of),
    object => lykia_native_fn!(nt_object_of),
    callable => lykia_native_fn!(nt_callable_of),
    tuple => lykia_native_fn!(nt_tuple_of)
}, {
    str => RV::Datatype(Datatype::Str),
    double => RV::Datatype(Datatype::Double),
    int32 => RV::Datatype(Datatype::Int32),
    int64 => RV::Datatype(Datatype::Int64),
    bool => RV::Datatype(Datatype::Bool),
    unit => RV::Datatype(Datatype::Unit),
    dtype => RV::Datatype(Datatype::Datatype),
    none => RV::Datatype(Datatype::None)
}, []);
