mod blocks;
mod functions;
mod ifs;
mod loops;

#[cfg(test)]

pub mod helpers {

    use std::rc::Rc;

    use rustc_hash::FxHashMap;

    use crate::runtime::environment::Environment;
    use crate::runtime::interpreter::{HaltReason, Interpreter};
    use crate::runtime::std::stdlib;
    use crate::runtime::types::{Function, Stateful, RV};
    use crate::runtime::{Runtime, RuntimeMode};
    use crate::util::{alloc_shared, Shared};

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

        let mut native_fns = stdlib();

        let mut test_namespace = FxHashMap::default();

        test_namespace.insert(
            "out".to_owned(),
            RV::Callable(None, Rc::new(Function::Stateful(out.clone()))),
        );

        native_fns.insert(
            "TestUtils".to_owned(),
            RV::Object(alloc_shared(test_namespace)),
        );

        for (name, value) in native_fns {
            env.borrow_mut().declare(name, value);
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
