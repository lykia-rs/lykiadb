use std::time;
use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{Callable, Reason, RV};

pub struct Clock;
impl Callable for Clock {
    fn arity(&self) -> Option<usize> {
        Some(0)
    }

    fn call(&self, _interpreter: &mut Interpreter, _args: Vec<RV>) -> Result<RV, Reason> {
        if let Ok(n) = time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
            return Err(Reason::Return(RV::Num(n.as_secs_f64())));
        }
        Err(Reason::Return(RV::Undefined))
    }
}