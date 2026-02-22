use lykiadb_lang::ast::Span;

use crate::{
    engine::interpreter::{HaltReason, InterpretError, Interpreter},
    lykia_module, lykia_native_fn,
    value::{RV, array::RVArray},
};

pub fn nt_create_arr<'v>(
    _interpreter: &mut Interpreter<'v>,
    called_from: &Span,
    args: &[RV<'v>],
) -> Result<RV<'v>, HaltReason<'v>> {
    let size = match &args[0] {
        RV::Double(n) if *n >= 0.0 && n.fract() == 0.0 => *n as usize,
        _ => {
            return Err(HaltReason::Error(
                InterpretError::InvalidArgumentType {
                    span: *called_from,
                    expected: "non-negative integer".to_string(),
                }
                .into(),
            ));
        }
    };

    // monotonically initialize array with undefined values
    let mut vec: Vec<RV> = Vec::with_capacity(size);
    for i in 0..size {
        vec.push(RV::Double(i as f64));
    }

    let arr = RV::Array(RVArray::from_vec(vec));
    Ok(arr)
}

lykia_module!(arr, {
    new => lykia_native_fn!(nt_create_arr)
}, {}, []);
