use crate::{
    error::ExecutionError,
    interpreter::{Interpreter, Output},
};
use lykiadb_common::memory::{Shared, alloc_shared};
use tracing::info;

use std::{collections::HashMap, sync::Arc};

use crate::value::RV;
use lykiadb_common::testing::TestHandler;
use lykiadb_lang::SourceProcessor;

pub struct Session<'v> {
    mode: SessionMode,
    source_processor: SourceProcessor,
    interpreter: Interpreter<'v>,
}

#[derive(Eq, PartialEq)]
pub enum SessionMode {
    Repl,
    File,
}

impl<'v> Session<'v> {
    pub fn new(mode: SessionMode, interpreter: Interpreter<'v>) -> Session<'v> {
        Session {
            mode,
            interpreter,
            source_processor: SourceProcessor::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<RV<'v>, ExecutionError> {
        let program = Arc::from(self.source_processor.process(source)?);
        let out = self.interpreter.interpret(program);

        if self.mode == SessionMode::Repl {
            info!("{:?}", out);
        }

        out
    }
}

pub struct SessionTester<'v> {
    out: Shared<Output<'v>>,
    session: Session<'v>,
}

impl<'v> Default for SessionTester<'v> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'v> SessionTester<'v> {
    pub fn new() -> SessionTester<'v> {
        let out = alloc_shared(Output::new());

        SessionTester {
            out: out.clone(),
            session: Session::new(SessionMode::File, Interpreter::new(Some(out), true)),
        }
    }
}

impl<'v> TestHandler for SessionTester<'v> {
    fn run_case(&mut self, case_parts: Vec<String>, flags: HashMap<&str, &str>) {
        assert!(
            case_parts.len() > 1,
            "Expected at least one input/output pair"
        );

        let mut errors: Vec<ExecutionError> = vec![];

        let result = self.session.interpret(&case_parts[0]);

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
                let result = self.session.interpret(stripped.trim());

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
