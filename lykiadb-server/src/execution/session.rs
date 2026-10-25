use crate::{
    database::Database, execution::{error::ExecutionError, state::ProgramState}, interpreter::{Interpreter, output::Output}
};
use lykiadb_common::memory::{Shared, alloc_shared};
use tracing::info;

use std::sync::Arc;

use crate::value::RV;
use lykiadb_common::testing::{Block, TestCase, TestFailure, TestHandler, dedent};
use lykiadb_lang::SourceProcessor;

pub struct Session<'v, D: Database> {
    keep_alive: bool,
    source_processor: SourceProcessor,
    program_state: Option<ProgramState<'v>>,
    db: D,
}

impl<'v, D: Database> Session<'v, D> {
    pub fn new(keep_alive: bool, db: D) -> Session<'v, D> {
        Session {
            keep_alive,
            source_processor: SourceProcessor::new(),
            program_state: None,
            db,
        }
    }

    pub fn interpret(
        &mut self,
        source: &str,
        out: Shared<Output<'v>>,
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

pub struct SessionTester<'v, D: Database> {
    out: Shared<Output<'v>>,
    session: Session<'v, D>,
}

impl<'v, D: Database> SessionTester<'v, D> {
    pub fn new(db: D) -> SessionTester<'v, D> {
        let out = alloc_shared(Output::new());

        SessionTester {
            out: out.clone(),
            session: Session::new(true, db),
        }
    }
}

impl<'v, D: Database> TestHandler for SessionTester<'v, D> {
    fn run_case(&mut self, case: TestCase) -> Result<(), TestFailure> {
        let mut errors: Vec<ExecutionError> = vec![];
        let mut last_returned: Option<RV> = None;

        for block in case.blocks {
            match block {
                Block::Input(code) => {
                    let returned = self.session.interpret(&dedent(&code), self.out.clone());

                    match returned {
                        Err(err) => {
                            errors.push(err);
                        }
                        Ok(val) => {
                            last_returned = Some(val);
                        }
                    };
                }
                Block::ExpectValue(raw) => {
                    if let Some(value) = last_returned {
                        pretty_assertions::assert_eq!(dedent(&raw), dedent(&value.to_string()));
                        last_returned = None;
                    } else {
                        panic!("There is no returned value")
                    }
                }
                Block::ExpectOutput(raw) => {
                    self.out.write().unwrap().expect(dedent(&raw))?;
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
    use crate::{database::simple::SimpleDatabase, interpreter::output::Output, storage::memory::MemoryStorageEngine};
    use lykiadb_common::memory::alloc_shared;

    #[test]
    fn repl_mode_interpret_logs_and_returns() {
        let out = alloc_shared(Output::new());
        let db = SimpleDatabase::new(MemoryStorageEngine::new());
        let mut session = Session::new(true, db);
        // A simple expression should succeed in Repl mode (exercises the info! branch)
        let result = session.interpret("1 + 1;", out);
        assert!(result.is_ok());
    }
}
