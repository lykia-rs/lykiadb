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
    keep_alive: bool,
    source_processor: SourceProcessor,
    program_state: Option<ProgramState<'v>>,
}

impl<'v> Session<'v> {
    pub fn new(keep_alive: bool) -> Session<'v> {
        Session {
            keep_alive,
            source_processor: SourceProcessor::new(),
            program_state: None,
        }
    }

    pub fn interpret(
        &mut self,
        source: &str,
        out: Option<Shared<Output<'v>>>,
    ) -> Result<RV<'v>, ExecutionError> {
        let program = Arc::from(self.source_processor.process(source)?);

        if let Some(state) = &self.program_state
            && self.keep_alive
        {
            self.program_state = Some(state.fork(out, program));
        } else {
            self.program_state = Some(ProgramState::new(out, program, true));
        }

        let mut interpreter = Interpreter::from_state(self.program_state.as_ref().unwrap());
        let res: Result<RV<'_>, ExecutionError> = interpreter.interpret();

        if self.keep_alive {
            info!("{:?}", res);
        } else {
            self.source_processor.reset();
        }

        res
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
            session: Session::new(true),
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
                    if let Err(err) = self
                        .session
                        .interpret(&dedent(&code), Some(self.out.clone()))
                    {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::output::Output;
    use lykiadb_common::memory::alloc_shared;

    #[test]
    fn repl_mode_interpret_logs_and_returns() {
        let out = alloc_shared(Output::new());
        let mut session = Session::new(true);
        // A simple expression should succeed in Repl mode (exercises the info! branch)
        let result = session.interpret("1 + 1;", Some(out));
        assert!(result.is_ok());
    }
}
