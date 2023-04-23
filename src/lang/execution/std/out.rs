use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{Callable, RV};

pub struct Print;

impl Print {
    pub fn new() -> Print {
        Print
    }
}

impl Callable for Print {
    fn arity(&self) -> u16 {
        0
    }

    fn call(&self, _interpreter: &Interpreter, args: Vec<RV>) -> RV {
        for arg in args {
            print!("{:?} ", arg);
        }
        println!();
        RV::Undefined
    }
}