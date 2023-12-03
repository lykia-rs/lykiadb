mod blocks;
mod functions;
mod ifs;
mod loops;

#[cfg(test)]

pub mod helpers {

    use crate::runtime::environment::Environment;
    use crate::runtime::interpreter::{HaltReason, Interpreter};
    use crate::runtime::std::fib::nt_fib;
    use crate::runtime::std::json::{nt_json_decode, nt_json_encode};
    use crate::runtime::std::time::nt_clock;
    use crate::runtime::types::{Function, Stateful, RV};
    use crate::runtime::{Runtime, RuntimeMode};
    use crate::util::{alloc_shared, Shared};
    use std::collections::HashMap;
    use std::rc::Rc;

    #[derive(Clone)]
    pub struct Output {
        out: Vec<RV>,
    }

    impl Output {
        pub fn new() -> Output {
            Output { out: Vec::new() }
        }

        pub fn push(&mut self, rv: RV) {
            self.out.push(rv);
        }

        pub fn expect(&mut self, rv: Vec<RV>) {
            assert_eq!(self.out, rv);
        }
    }

    impl Stateful for Output {
        fn call(&mut self, _interpreter: &mut Interpreter, rv: &[RV]) -> Result<RV, HaltReason> {
            for item in rv {
                self.push(item.clone());
            }
            Ok(RV::Undefined)
        }
    }

    pub fn get_runtime() -> (Shared<Output>, Runtime) {
        let env = Environment::new(None);

        let out = alloc_shared(Output::new());

        let native_fns = HashMap::from([
            (
                "clock",
                RV::Callable(Some(0), Rc::new(Function::Lambda { function: nt_clock })),
            ),
            (
                "print",
                RV::Callable(None, Rc::new(Function::Stateful(out.clone()))),
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

        (
            out,
            Runtime {
                env,
                mode: RuntimeMode::File,
            },
        )
    }

    pub fn exec_assert(code: &str, output: Vec<RV>) -> () {
        let (out, mut runtime) = get_runtime();
        runtime.interpret(code);
        out.borrow_mut().expect(output);
    }
}
