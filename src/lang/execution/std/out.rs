use crate::lang::execution::interpreter::Interpreter;
use crate::lang::parsing::types::{RV, CallableError};

pub fn nt_print(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, CallableError> {
    for arg in args {
        print!("{:?} ", arg);
    }
    println!();
    Ok(RV::Undefined)
}