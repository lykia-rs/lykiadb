use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use crate::lang::execution::interpreter::Interpreter;

#[derive(Debug, Clone)]
pub enum RV {
    Str(Rc<String>),
    Num(f64),
    Bool(bool),
    Undefined,
    NaN,
    Nil,
    Callable(Rc<dyn Callable>)
}

pub trait Callable {
    fn arity(&self) -> u16;
    fn call(&self, interpreter: &Interpreter, args: Vec<RV>) -> RV;
}

impl Debug for dyn Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Callable>")
    }
}