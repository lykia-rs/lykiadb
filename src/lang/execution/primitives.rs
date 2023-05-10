use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use crate::lang::execution::interpreter::Interpreter;
use crate::lang::parsing::ast::{Stmt};

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
    fn arity(&self) -> Option<usize>;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<RV>) -> RV;
}

impl Debug for dyn Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

pub struct Function {
    pub parameters: Vec<Rc<String>>,
    pub body: Rc<Vec<Stmt>>,
}

impl Callable for Function {
    fn arity(&self) -> Option<usize> {
        Some(self.parameters.len())
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<RV>) -> RV {
        interpreter.execute_block(&self.body)
    }
}