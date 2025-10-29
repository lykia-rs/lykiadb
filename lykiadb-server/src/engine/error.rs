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
            ExecutionError::Lang(LangError::Scan(ScanError::UnexpectedCharacter { span })) => {
                StandardError::new("Unexpected character", "Remove this character", to_error_span(span))
            }
            ExecutionError::Lang(LangError::Scan(ScanError::UnterminatedString { span })) => {
                StandardError::new(
                    "Unterminated string",
                    "Terminate the string with a double quote (\").",
                    to_error_span(span),
                )
            }
            ExecutionError::Lang(LangError::Scan(ScanError::MalformedNumberLiteral { span })) => {
                StandardError::new(
                    "Malformed number literal",
                    "Make sure that number literal is up to spec.",
                    to_error_span(span),
                )
            }
            ExecutionError::Lang(LangError::Parse(ParseError::MissingToken { token, expected })) => {
                StandardError::new(
                    "Missing token",
                    &format!(
                        "Add a {:?} token after \"{}\".",
                        expected,
                        token.lexeme.unwrap()
                    ),
                    to_error_span(token.span),
                )
            }
            ExecutionError::Lang(LangError::Parse(ParseError::NoTokens)) => {
                StandardError::new("There is nothing to parse", "", to_error_span(Span::default()))
            }
            ExecutionError::Lang(LangError::Parse(ParseError::InvalidAssignmentTarget { left })) => {
                StandardError::new(
                    "Invalid assignment target",
                    &format!("No values can be assigned to {}", left.lexeme.unwrap()),
                    to_error_span(left.span),
                )
            }
            ExecutionError::Lang(LangError::Parse(ParseError::UnexpectedToken { token })) => {
                StandardError::new(
                    "Unexpected token",
                    &format!(
                        "Unexpected token {}.",
                        token.lexeme.unwrap_or("None".to_string())
                    ),
                    to_error_span(token.span),
                )
            }
            ExecutionError::Interpret(InterpretError::UnexpectedStatement { span }) => {
                StandardError::new("Unexpected statement", "Remove this.", to_error_span(span))
            }
            ExecutionError::Interpret(InterpretError::NotCallable { span }) => {
                StandardError::new(
                    "Not callable",
                    "Expression does not yield a callable.",
                    to_error_span(span),
                )
            }
            ExecutionError::Interpret(InterpretError::PropertyNotFound { property, span }) => {
                StandardError::new(
                    &format!("Property {property} not found in the evaluated expression"),
                    "Check if that field is present in the expression.",
                    to_error_span(span),
                )
            }
            ExecutionError::Plan(planner_error) => planner_error.into(),
            ExecutionError::Environment(EnvironmentError::Other { message })
            | ExecutionError::Interpret(InterpretError::Other { message }) => {
                StandardError::new(&message, "", to_error_span(Span::default()))
            }
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
