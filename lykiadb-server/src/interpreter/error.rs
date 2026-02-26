use lykiadb_common::error::InputError;
use lykiadb_lang::ast::Span;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum InterpretError {
    #[error("Expression is not callable at {span:?}")]
    NotCallable { span: Span },
    #[error("Unexpected statement at {span:?}")]
    UnexpectedStatement { span: Span },
    #[error("Property '{property}' not found at {span:?}")]
    PropertyNotFound { span: Span, property: String },
    #[error("Only select expressions can be explained.")]
    InvalidExplainTarget { span: Span },
    #[error("Range can only be created with numbers.")]
    InvalidRangeExpression { span: Span },
    #[error("Only objects have properties.")]
    InvalidPropertyAccess { span: Span, value_str: String },
    #[error("Argument type mismatch. Expected {expected:?}")]
    InvalidArgumentType { span: Span, expected: String },
    #[error("No program loaded in interpreter.")]
    NoProgramLoaded,
}

impl From<InterpretError> for InputError {
    fn from(value: InterpretError) -> Self {
        let (hint, sp) = match &value {
            InterpretError::NotCallable { span } => (
                "Ensure the expression evaluates to a callable function",
                *span,
            ),
            InterpretError::UnexpectedStatement { span } => (
                "Check if the statement is used in the correct context",
                *span,
            ),
            InterpretError::PropertyNotFound { span, .. } => {
                ("Verify the property name exists on the object", *span)
            }
            InterpretError::InvalidExplainTarget { span, .. } => {
                ("Try replacing this with a SELECT expression", *span)
            }
            InterpretError::InvalidRangeExpression { span } => (
                "Ensure that the range expression is built with numbers",
                *span,
            ),
            InterpretError::InvalidPropertyAccess { span, value_str } => (
                &format!(
                    "Ensure that the highlighted expression evaluates to an object: {value_str}"
                ) as &str,
                *span,
            ),
            InterpretError::InvalidArgumentType { span, .. } => {
                ("Check that the argument matches the expected types", *span)
            }
            InterpretError::NoProgramLoaded => (
                "Load a program into the interpreter before execution",
                Span::default(),
            ),
        };

        InputError::new(&value.to_string(), hint, Some(sp.into()))
    }
}
