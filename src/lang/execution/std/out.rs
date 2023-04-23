use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{Callable, RV};

pub struct Print;
impl Callable for Print {
    fn arity(&self) -> Option<usize> {
        None
    }

    fn call(&self, _interpreter: &mut Interpreter, args: Vec<RV>) -> RV {
        for arg in args {
            print!("{:?} ", arg);
        }
        println!();
        RV::Undefined
    }
}