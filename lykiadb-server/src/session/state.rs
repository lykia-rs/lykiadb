use crate::interpreter::environment::EnvironmentFrame;
use crate::interpreter::output::Output;
use crate::value::iterator::ExecutionRow;
use lykiadb_common::memory::Shared;
use std::sync::Arc;

use lykiadb_lang::parser::program::Program;
#[derive(Clone)]
pub struct ProgramState<'sess> {
    pub env: Arc<EnvironmentFrame<'sess>>,
    pub exec_row: Shared<Option<ExecutionRow<'sess>>>,
    // Output
    pub output: Option<Shared<Output<'sess>>>,
    // Static fields:
    pub root_env: Arc<EnvironmentFrame<'sess>>,
    pub program: Option<Arc<Program>>,
}

impl<'sess> ProgramState<'sess> {
    pub fn new(
        env: Arc<EnvironmentFrame<'sess>>,
        root_env: Arc<EnvironmentFrame<'sess>>,
        program: Option<Arc<Program>>,
        output: Option<Shared<Output<'sess>>>,
    ) -> Self {
        Self {
            env,
            root_env,
            exec_row: Shared::new(None.into()),
            program,
            output,
        }
    }
}
