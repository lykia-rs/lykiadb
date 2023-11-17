use super::interpreter::{HaltReason, Interpreter};
use super::types::Stateful;
use super::{Runtime, RuntimeMode};
use crate::runtime::environment::Environment;
use crate::runtime::std::fib::nt_fib;
use crate::runtime::std::json::{nt_json_decode, nt_json_encode};
use crate::runtime::std::out::nt_print;
use crate::runtime::std::time::nt_clock;
use crate::runtime::types::{Function, RV};
use crate::util::alloc_shared;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
struct Output {
    out: Vec<RV>,
    expected: Vec<RV>,
}

impl Output {
    fn new() -> Output {
        Output {
            out: Vec::new(),
            expected: Vec::new(),
        }
    }

    pub fn push(&mut self, rv: RV) {
        self.out.push(rv);
    }

    fn expect(&mut self, rv: RV) {
        self.expected.push(rv);
    }

    fn assert(&self) {
        assert_eq!(self.out, self.expected);
    }
}

impl Stateful for Output {
    fn call(&mut self, interpreter: &mut Interpreter, rv: &[RV]) -> Result<RV, HaltReason> {
        for item in rv {
            self.push(item.clone());
        }
        Ok(RV::Undefined)
    }
}

pub fn get_runtime() -> Runtime {
    let env = Environment::new(None);

    let out = alloc_shared(Output::new());

    let native_fns = HashMap::from([
        (
            "collect",
            RV::Callable(Some(0), Rc::new(Function::Stateful(out))),
        ),
        (
            "clock",
            RV::Callable(Some(0), Rc::new(Function::Lambda { function: nt_clock })),
        ),
        (
            "print",
            RV::Callable(None, Rc::new(Function::Lambda { function: nt_print })),
        ),
        (
            "fib_nat",
            RV::Callable(Some(1), Rc::new(Function::Lambda { function: nt_fib })),
        ),
        (
            "json_encode",
            RV::Callable(
                Some(1),
                Rc::new(Function::Lambda {
                    function: nt_json_encode,
                }),
            ),
        ),
        (
            "json_decode",
            RV::Callable(
                Some(1),
                Rc::new(Function::Lambda {
                    function: nt_json_decode,
                }),
            ),
        ),
    ]);

    for (name, value) in native_fns {
        env.borrow_mut().declare(name.to_string(), value);
    }

    Runtime {
        env,
        mode: RuntimeMode::File,
    }
}
