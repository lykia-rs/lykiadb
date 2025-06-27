use crate::{
    engine::interpreter::{HaltReason, Interpreter},
    value::RV,
};

pub fn nt_print(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    for arg in args {
        print!("{arg:?} ");
    }
    println!();
    Ok(RV::Undefined)
}
