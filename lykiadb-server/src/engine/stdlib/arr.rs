use crate::{
    engine::interpreter::{HaltReason, InterpretError, Interpreter},
    value::{RV, array::RVArray},
};

pub fn nt_create_arr(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    let size = match &args[0] {
        RV::Num(n) if *n >= 0.0 && n.fract() == 0.0 => *n as usize,
        _ => {
            return Err(HaltReason::Error(
                InterpretError::Other {
                    message: format!(
                        "arr::new: Expected non-negative integer size, got '{:?}'",
                        args[0]
                    ),
                }
                .into(),
            ));
        }
    };

    // monotonically initialize array with undefined values
    let mut vec: Vec<RV> = Vec::with_capacity(size);
    for i in 0..size {
        vec.push(RV::Num(i as f64));
    }

    let arr = RV::Array(RVArray::from_vec(vec));
    Ok(arr)
}
