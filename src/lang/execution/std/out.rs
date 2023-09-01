use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{HaltReason};
use crate::lang::parsing::token::RV;

pub fn nt_print(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    for arg in args {
        print!("{:?} ", arg);
    }
    println!();
    Ok(RV::Undefined)
}