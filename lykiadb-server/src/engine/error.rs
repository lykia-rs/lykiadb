use std::fmt::{Display, Formatter, Result};

use crate::{plan::PlannerError, value::environment::EnvironmentError};

use super::interpreter::InterpretError;
use lykiadb_common::error::StandardError;
use lykiadb_lang::{LangError, ast::Span, parser::ParseError, tokenizer::scanner::ScanError};
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

pub fn to_error_span(span: Span) -> Option<lykiadb_common::error::Span> {
    Some(lykiadb_common::error::Span {
        start: span.start,
        end: span.end,
        line: span.line,
        line_end: span.line_end,
    })
}

impl ExecutionError {
    pub fn generalize(
        self,
    ) -> StandardError {
        match self {
            ExecutionError::Lang(lang_error) => lang_error.into(),
            ExecutionError::Interpret(interpret_error) => interpret_error.into(),
            ExecutionError::Plan(planner_error) => planner_error.into(),
            ExecutionError::Environment(env_error) => env_error.into(),
            _ => {
                StandardError::new(
                    "Unknown error",
                    "An unknown error has occurred.",
                    to_error_span(Span::default()),
                )
            }
        }
    }
}
