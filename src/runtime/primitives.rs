use std::fmt::{Debug, Formatter, Display};
use std::process::exit;
use std::rc::Rc;
use crate::lang::types::{CallableError, Callable};
use crate::runtime::environment::{Environment, Shared};
use crate::runtime::interpreter::Interpreter;
use crate::lang::ast::Stmt;
use crate::lang::types::RV;
pub enum Function {
    Native(fn(&mut Interpreter, &[RV]) -> Result<RV, CallableError>),
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

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Native(_) => write!(f, "<native_fn>"),
            Function::UserDefined(name, _, _, _) => write!(f, "{}", name),
            _ => exit(1)
        }
    }
}

impl Callable for Function {
    fn call(&self, interpreter: &mut Interpreter, arguments: &[RV]) -> Result<RV, CallableError> {
        match self {
            Function::Native(function) => function(interpreter, arguments),
            Function::UserDefined(_name, body, parameters, closure) => {
                let fn_env = Environment::new(Some(Rc::clone(closure)));

                for (i, param) in parameters.iter().enumerate() {
                    fn_env.borrow_mut().declare(param.to_string(), arguments.get(i).unwrap().clone());
                }

                interpreter.user_fn_call(body, fn_env)
            }
            _ => exit(1)
        }
    }
}
