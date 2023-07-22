use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{Callable, Reason, RV};

pub struct Print;
impl Callable for Print {
    fn arity(&self) -> Option<usize> {
        None
    }

    fn call(&self, _interpreter: &mut Interpreter, args: Vec<RV>) -> Result<RV, Reason> {
        for arg in args {
            print!("{:?} ", arg);
        }
        println!();
        Err(Reason::Return(RV::Undefined))
    }
}