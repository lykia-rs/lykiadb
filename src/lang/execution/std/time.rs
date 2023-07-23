use std::io::repeat;
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

pub struct Bench;

impl Callable for Bench {
    fn arity(&self) -> Option<usize> {
        Some(2)
    }

    fn call(&self, _interpreter: &mut Interpreter, _args: Vec<RV>) -> Result<RV, Reason> {
        let benched = &_args[0];
        let repeats = &_args[1];

        if let RV::Callable(benched_unwrapped) = benched {
            if let RV::Num(repeat_unwrapped) = repeats {
                let repeat_int = *repeat_unwrapped as i32;
                let mut total: f64 = 0f64;

                for _ in 0..repeat_int {
                    let start =  time::SystemTime::now().duration_since(time::UNIX_EPOCH);
                    benched_unwrapped.call(_interpreter, Vec::new())?;
                    let end =  time::SystemTime::now().duration_since(time::UNIX_EPOCH);
                    total += end.unwrap().as_secs_f64() - start.unwrap().as_secs_f64();
                }
                return Err(Reason::Return(RV::Num(total / repeat_int as f64)));
            }
        }
        Err(Reason::Error("Unexpected types for bench function".to_owned()))
    }
}