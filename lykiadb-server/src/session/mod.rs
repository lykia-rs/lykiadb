use crate::{
    execution::{error::ExecutionError, state::ProgramState},
    interpreter::{Interpreter, output::Output},
};
use lykiadb_common::memory::{Shared, alloc_shared};
use tracing::info;

use std::sync::Arc;

use crate::value::RV;
use lykiadb_common::testing::{Block, TestCase, TestFailure, TestHandler, dedent};
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
            session: Session::new(
                SessionMode::File,
                Interpreter::from_state(&ProgramState::new(Some(out), true)),
            ),
        }
    }
}

impl<'v> TestHandler for SessionTester<'v> {
    fn run_case(&mut self, case: TestCase) -> Result<(), TestFailure> {
        let run_mode = case.flags.get("run").map(|s| s.as_str()).unwrap_or("");
        let mut errors: Vec<ExecutionError> = vec![];

        for block in case.blocks {
            match block {
                Block::Input(code) => {
                    if let Err(err) = self.session.interpret(&dedent(&code)) {
                        errors.push(err);
                    }
                }
                Block::Expect(raw) => {
                    let expected = dedent(&raw);
                    if run_mode == "plan" {
                        self.out
                            .write()
                            .unwrap()
                            .expect(vec![RV::Str(Arc::new(expected))])?;
                    } else {
                        let lines: Vec<String> =
                            expected.split('\n').map(|l| l.to_string()).collect();
                        self.out.write().unwrap().expect_str(lines)?;
                    }
                }
                Block::ExpectErr(raw) => {
                    let expected = dedent(&raw);
                    let actual = errors
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join("\n");
                    if actual != expected {
                        return Err(crate::interpreter::output::str_diff(&actual, &expected));
                    }
                }
            }
        }
        Ok(())
    }
}
