use std::fmt::{Display, Formatter, Result};

use crate::{interpreter::error::InterpretError, query::plan::error::PlannerError, value::environment::EnvironmentError};

use lykiadb_common::error::InputError;
use lykiadb_lang::LangError;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    Lang(LangError),
    Interpret(InterpretError),
    Environment(EnvironmentError),
    Plan(PlannerError),
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{self:?}")
    }
}

impl ExecutionError {
    pub fn generalize(self) -> InputError {
        match self {
            ExecutionError::Lang(lang_error) => lang_error.into(),
            ExecutionError::Interpret(interpret_error) => interpret_error.into(),
            ExecutionError::Plan(planner_error) => planner_error.into(),
            ExecutionError::Environment(env_error) => env_error.into(),
        }
    }
}

impl From<LangError> for ExecutionError {
    fn from(err: LangError) -> Self {
        ExecutionError::Lang(err)
    }
}

impl From<InterpretError> for ExecutionError {
    fn from(err: InterpretError) -> Self {
        ExecutionError::Interpret(err)
    }
}