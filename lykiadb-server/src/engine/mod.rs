use self::error::ExecutionError;
use crate::value::RV;
use interpreter::Interpreter;
use tracing::info;

pub mod error;
pub mod interpreter;
mod stdlib;

pub struct Runtime {
    mode: RuntimeMode,
    interpreter: Interpreter,
}

#[derive(Eq, PartialEq)]
pub enum RuntimeMode {
    Repl,
    File,
}

impl Runtime {
    pub fn new(mode: RuntimeMode, interpreter: Interpreter) -> Runtime {
        Runtime { mode, interpreter }
    }

    pub fn interpret(&mut self, source: &str) -> Result<RV, ExecutionError> {
        let out = self.interpreter.interpret(source);

        if self.mode == RuntimeMode::Repl {
            info!("{:?}", out);
        }

        out
    }
}

pub mod test_helpers {
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::engine::{error::ExecutionError, Interpreter, Runtime, RuntimeMode};
    use crate::util::{alloc_shared, Shared};
    use crate::value::RV;

    use super::interpreter::Output;

    pub struct RuntimeTester {
        out: Shared<Output>,
        runtime: Runtime,
    }

    impl Default for RuntimeTester {
        fn default() -> Self {
            Self::new()
        }
    }

    impl RuntimeTester {
        pub fn new() -> RuntimeTester {
            let out = alloc_shared(Output::new());

            RuntimeTester {
                out: out.clone(),
                runtime: Runtime::new(RuntimeMode::File, Interpreter::new(Some(out), true)),
            }
        }

        pub fn test_file(input: &str) {
            let parts: Vec<&str> = input.split("#[").collect();

            for part in parts[1..].iter() {
                let mut tester = RuntimeTester::new();

                let directives_and_input = part.trim();

                let directives_end = directives_and_input
                    .find('>')
                    .unwrap_or(directives_and_input.len());

                let rest = directives_and_input[directives_end + 1..]
                    .trim()
                    .to_string();

                let flags = directives_and_input[..directives_end - 1]
                    .trim()
                    .split(',')
                    .map(|flag| {
                        let kv: Vec<&str> = flag.split('=').collect();
                        (kv[0].trim(), kv[1].trim())
                    })
                    .fold(std::collections::HashMap::new(), |mut acc, (k, v)| {
                        acc.insert(k, v);
                        acc
                    });

                let case_parts = rest.split("---").map(|x| x.trim().to_string()).collect();

                match flags.get("run") {
                    Some(&"plan") | Some(&"interpreter") => {
                        tester.run_case(case_parts, flags.clone());
                    }
                    _ => panic!("Unknown directive"),
                }
            }
        }

        fn run_case(&mut self, case_parts: Vec<String>, flags: HashMap<&str, &str>) {
            assert!(
                case_parts.len() > 1,
                "Expected at least one input/output pair"
            );

            let mut errors: Vec<ExecutionError> = vec![];

            let result = self.runtime.interpret(&case_parts[0]);

            if let Err(err) = result {
                errors.push(err);
            }

            for part in &case_parts[1..] {
                if let Some(stripped) = part.strip_prefix("err") {
                    assert_eq!(
                        errors
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join("\n"),
                        stripped.trim()
                    );
                } else if let Some(stripped) = part.strip_prefix('>') {
                    let result = self.runtime.interpret(stripped.trim());

                    if let Err(err) = result {
                        errors.push(err);
                    }
                } else if flags.get("run") == Some(&"plan") {
                    // TODO(vck): Remove this
                    self.out
                        .write()
                        .unwrap()
                        .expect(vec![RV::Str(Arc::new(part.to_string()))]);
                } else {
                    self.out
                        .write()
                        .unwrap()
                        .expect_str(part.split('\n').map(|x| x.to_string()).collect());
                }
            }
        }
    }
}
