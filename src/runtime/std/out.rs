use crate::runtime::interpreter::Interpreter;
use crate::lang::types::{RV, CallableError};

pub fn nt_print(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, CallableError> {
    for arg in args {
        print!("{:?} ", arg);
    }
    println!();
    Ok(RV::Undefined)
}