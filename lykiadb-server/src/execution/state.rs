use crate::execution::global::GLOBAL_INTERNER;
use crate::interpreter::environment::{EnvironmentFrame, EnvironmentOrigin};
use crate::interpreter::output::Output;
use crate::libs::stdlib::stdlib;
use lykiadb_common::memory::Shared;
use std::sync::Arc;

use lykiadb_lang::parser::program::Program;
#[derive(Clone)]
pub struct ProgramState<'sess> {
    pub root_env: Arc<EnvironmentFrame<'sess>>,
    pub env: Arc<EnvironmentFrame<'sess>>,
    // Output
    pub output: Shared<Output<'sess>>,
    // Static fields:
    pub program: Arc<Program>,
}

impl<'sess> ProgramState<'sess> {
    pub fn new(
        output: Shared<Output<'sess>>,
        program: Arc<Program>,
        with_stdlib: bool,
    ) -> ProgramState<'sess> {
        let root_env = Arc::new(EnvironmentFrame::new(None, EnvironmentOrigin::Root));
        if with_stdlib {
            let native_fns = stdlib();

            for (name, value) in native_fns {
                root_env.define(GLOBAL_INTERNER.intern(&name), value);
            }
        }

        ProgramState {
            env: root_env.clone(),
            root_env: root_env.clone(),
            program,
            output,
        }
    }

    pub fn fork(
        &self,
        output: Shared<Output<'sess>>,
        program: Arc<Program>,
    ) -> ProgramState<'sess> {
        ProgramState {
            env: Arc::clone(&self.env),
            root_env: Arc::clone(&self.root_env),
            program,
            output,
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    use lykiadb_common::memory::alloc_shared;

    use super::*;

    pub fn create_empty_state<'sess>() -> ProgramState<'sess> {
        ProgramState::new(alloc_shared(Output::new()), Arc::new(Program::empty()), true)
    }
}
