use std::fmt::{Display, Formatter, Result};

use crate::{plan::PlannerError, value::environment::EnvironmentError};

use super::interpreter::InterpretError;
use lykiadb_common::comm::StandardErrorImpl;
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

fn to_error_span(span: Span) -> Option<lykiadb_common::error::Span> {
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
    ) -> StandardErrorImpl {
        match self {
            ExecutionError::Lang(LangError::Scan(ScanError::UnexpectedCharacter { span })) => {
                StandardErrorImpl::new("Unexpected character", "Remove this character", to_error_span(span))
            }
            ExecutionError::Lang(LangError::Scan(ScanError::UnterminatedString { span })) => {
                StandardErrorImpl::new(
                    "Unterminated string",
                    "Terminate the string with a double quote (\").",
                    to_error_span(span),
                )
            }
            ExecutionError::Lang(LangError::Scan(ScanError::MalformedNumberLiteral { span })) => {
                StandardErrorImpl::new(
                    "Malformed number literal",
                    "Make sure that number literal is up to spec.",
                    to_error_span(span),
                )
            }
            ExecutionError::Lang(LangError::Parse(ParseError::MissingToken { token, expected })) => {
                StandardErrorImpl::new(
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
                StandardErrorImpl::new("There is nothing to parse", "", to_error_span(Span::default()))
            }
            ExecutionError::Lang(LangError::Parse(ParseError::InvalidAssignmentTarget { left })) => {
                StandardErrorImpl::new(
                    "Invalid assignment target",
                    &format!("No values can be assigned to {}", left.lexeme.unwrap()),
                    to_error_span(left.span),
                )
            }
            ExecutionError::Lang(LangError::Parse(ParseError::UnexpectedToken { token })) => {
                StandardErrorImpl::new(
                    "Unexpected token",
                    &format!(
                        "Unexpected token {}.",
                        token.lexeme.unwrap_or("None".to_string())
                    ),
                    to_error_span(token.span),
                )
            }
            ExecutionError::Interpret(InterpretError::UnexpectedStatement { span }) => {
                StandardErrorImpl::new("Unexpected statement", "Remove this.", to_error_span(span))
            }
            ExecutionError::Interpret(InterpretError::NotCallable { span }) => {
                StandardErrorImpl::new(
                    "Not callable",
                    "Expression does not yield a callable.",
                    to_error_span(span),
                )
            }
            ExecutionError::Interpret(InterpretError::PropertyNotFound { property, span }) => {
                StandardErrorImpl::new(
                    &format!("Property {property} not found in the evaluated expression"),
                    "Check if that field is present in the expression.",
                    to_error_span(span),
                )
            }
            ExecutionError::Plan(PlannerError::DuplicateObjectInScope(previous)) => {
                StandardErrorImpl::new(
                    "Duplicate object in scope",
                    &format!("Object {} is already defined in the scope.", previous.name),
                    to_error_span(previous.span),
                )
            }
            ExecutionError::Plan(PlannerError::SubqueryNotAllowed(span)) => {
                StandardErrorImpl::new(
                    "Subquery not allowed",
                    "Subqueries are not allowed in this context.",
                    to_error_span(span),
                )
            }
            ExecutionError::Plan(PlannerError::NestedAggregationNotAllowed(span)) => {
                StandardErrorImpl::new(
                    "Aggregate function cannot be nested inside another aggregate function",
                    "Remove inner aggregation.",
                    to_error_span(span),
                )
            }
            ExecutionError::Plan(PlannerError::AggregationNotAllowed(span, context)) => {
                StandardErrorImpl::new(
                    &format!("Aggregation not allowed in {context}"),
                        "Remove aggregation.",
                    to_error_span(span),
                )
            }
            ExecutionError::Plan(PlannerError::HavingWithoutAggregationNotAllowed(span)) => {
                StandardErrorImpl::new(
                    "HAVING clause without aggregation is not allowed",
                    "Add aggregation or remove HAVING clause.",
                    to_error_span(span),
                )
            }
            ExecutionError::Environment(EnvironmentError::Other { message })
            | ExecutionError::Interpret(InterpretError::Other { message }) => {
                StandardErrorImpl::new(&message, "", to_error_span(Span::default()))
            }
            _ => {
                StandardErrorImpl::new(
                    "Unknown error",
                    "An unknown error has occurred.",
                    to_error_span(Span::default()),
                )
            }
        }
    }
}
