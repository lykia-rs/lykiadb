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
    fn get_desc(&self) -> &str {
        "<native_fn>"
    }
}

impl Debug for dyn Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_desc())
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
        let parameters = &self.parameters;

        let mut pairs: Vec<(String, RV)> = Vec::new();

        for (i, param) in parameters.iter().enumerate() {
            pairs.push((param.to_string(), args.get(i).unwrap().clone()));
        }

        interpreter.user_fn_call(&self.body, Some(pairs))

    }

    fn get_desc(&self) -> &str {
        "<fn>"
    }
}