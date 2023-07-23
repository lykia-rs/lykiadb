use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use crate::lang::execution::environment::{Environment, Shared};
use crate::lang::execution::interpreter::Interpreter;
use crate::lang::parsing::ast::Stmt;

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

#[derive(Debug)]
pub enum HaltReason {
    Error(String),
    Return(RV),
}

pub fn runtime_err(msg: &str, line: u32) -> HaltReason {
    HaltReason::Error(format!("{} at line {}", msg, line + 1))
}

pub trait Callable {
    fn arity(&self) -> Option<usize>;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<RV>) -> Result<RV, HaltReason>;
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
    pub closure: Option<Shared<Environment>>
}

impl Callable for Function {
    fn arity(&self) -> Option<usize> {
        Some(self.parameters.len())
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<RV>) -> Result<RV, HaltReason> {
        let parameters = &self.parameters;
        let fn_env = Environment::new(self.closure.clone());

        for (i, param) in parameters.iter().enumerate() {
            fn_env.borrow_mut().declare(param.to_string(), args.get(i).unwrap().clone());
        }

        interpreter.user_fn_call(&self.body, fn_env).map(|_| RV::Undefined)
    }

    fn get_desc(&self) -> &str {
        "<fn>"
    }
}