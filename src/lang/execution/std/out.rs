use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{Callable, HaltReason, RV};

pub struct Print;
impl Callable for Print {
    fn arity(&self) -> Option<usize> {
        None
    }

    fn call(&self, _interpreter: &mut Interpreter, args: Vec<RV>) -> Result<RV, HaltReason> {
        for arg in args {
            print!("{:?} ", arg);
        }
        println!();
        Ok(RV::Undefined)
    }
}