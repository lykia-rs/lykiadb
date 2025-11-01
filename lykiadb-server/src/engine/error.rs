use std::fmt::{Display, Formatter, Result};

use crate::{plan::PlannerError, value::environment::EnvironmentError};

use super::interpreter::InterpretError;
use lykiadb_common::error::StandardError;
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
    pub fn generalize(self) -> StandardError {
        match self {
            ExecutionError::Lang(lang_error) => lang_error.into(),
            ExecutionError::Interpret(interpret_error) => interpret_error.into(),
            ExecutionError::Plan(planner_error) => planner_error.into(),
            ExecutionError::Environment(env_error) => env_error.into(),
        }
    }
}
