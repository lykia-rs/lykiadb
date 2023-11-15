use crate::runtime::environment::Environment;
use crate::runtime::std::fib::nt_fib;
use crate::runtime::std::json::{nt_json_decode, nt_json_encode};
use crate::runtime::std::out::{nt_print, self};
use crate::runtime::std::time::nt_clock;
use crate::runtime::types::{Function, RV};
use std::collections::HashMap;
use ::std::rc::Rc;
use super::{Runtime, RuntimeMode};
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

pub fn get_runtime() -> Runtime {
    let env = Environment::new(None);

    let mut out = Output::new();

    let native_fns = HashMap::from([
        (
            "clock",
            RV::Callable(Some(0), Rc::new(Function::Native { function: nt_clock })),
        ),
        (
            "print",
            RV::Callable(None, Rc::new(Function::Native { function: nt_print })),
        ),
        (
            "fib_nat",
            RV::Callable(Some(1), Rc::new(Function::Native { function: nt_fib })),
        ),
        (
            "json_encode",
            RV::Callable(
                Some(1),
                Rc::new(Function::Native {
                    function: nt_json_encode,
                }),
            ),
        ),
        (
            "json_decode",
            RV::Callable(
                Some(1),
                Rc::new(Function::Native {
                    function: nt_json_decode,
                }),
            ),
        ),
    ]);

    for (name, value) in native_fns {
        env.borrow_mut().declare(name.to_string(), value);
    }

    Runtime { env, mode: RuntimeMode::File }
}
