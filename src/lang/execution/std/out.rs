use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{Callable, HaltReason, RV};

pub fn nt_print(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, HaltReason> {
    for arg in args {
        print!("{:?} ", arg);
    }
    println!();
    Ok(RV::Undefined)
}