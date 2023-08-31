use std::fmt::{Debug, Formatter};
use std::process::exit;
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
    Callable(Option<usize>, Rc<Function>)
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

pub enum Function {
    Native(fn(&mut Interpreter, &[RV]) -> Result<RV, HaltReason>),
    UserDefined(String, Rc<Vec<Stmt>>, Vec<String>, Shared<Environment>),
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Native(_) => write!(f, "<native_fn>"),
            Function::UserDefined(name, _, _, _) => write!(f, "{}", name),
            _ => exit(1)
        }
    }
}

impl Function {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: &[RV]) -> Result<RV, HaltReason> {
        match self {
            Function::Native(function) => function(interpreter, arguments),
            Function::UserDefined(_name, body, parameters, closure) => {
                let fn_env = Environment::new(Some(Rc::clone(closure)));

                for (i, param) in parameters.iter().enumerate() {
                    fn_env.borrow_mut().declare(param.to_string(), arguments.get(i).unwrap().clone());
                }

                interpreter.user_fn_call(&body, fn_env).map(|_| RV::Undefined)
            }
            _ => exit(1)
        }
    }
}
