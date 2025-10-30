use crate::{
    engine::interpreter::{HaltReason, Interpreter},
    value::StdVal,
};

pub fn nt_print(_interpreter: &mut Interpreter, args: &[StdVal]) -> Result<StdVal, HaltReason> {
    for arg in args {
        print!("{arg:?} ");
    }
    println!();
    Ok(StdVal::Undefined)
}
