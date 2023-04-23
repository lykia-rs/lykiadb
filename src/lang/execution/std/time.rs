use std::time;
use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{Callable, RV};

pub struct Clock;
impl Callable for Clock {
    fn arity(&self) -> Option<usize> {
        Some(0)
    }

    fn call(&self, _interpreter: &Interpreter, _args: Vec<RV>) -> RV {
        if let Ok(n) = time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
            return RV::Num(n.as_secs_f64());
        }
        RV::Undefined
    }
}