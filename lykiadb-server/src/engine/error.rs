use std::fmt::{Display, Formatter, Result};

use crate::{plan::PlannerError, value::environment::EnvironmentError};

use super::interpreter::InterpretError;
use lykiadb_lang::{
    parser::{resolver::ResolveError, ParseError},
    tokenizer::scanner::ScanError,
    Span,
};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    Scan(ScanError),
    Parse(ParseError),
    Resolve(ResolveError),
    Interpret(InterpretError),
    Environment(EnvironmentError),
    Plan(PlannerError),
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl From<ParseError> for ExecutionError {
    fn from(err: ParseError) -> Self {
        ExecutionError::Parse(err)
    }
}

impl From<ScanError> for ExecutionError {
    fn from(err: ScanError) -> Self {
        ExecutionError::Scan(err)
    }
}

pub fn report_error(source_name: &str, source: &str, error: ExecutionError, mut writer: impl std::io::Write) {
    use ariadne::{Color, Label, Report, ReportKind, Source};

    // Generate & choose some colours for each of our elements
    let out = Color::Fixed(81);

    let mut print = |message: &str, hint: &str, span: Span| {
        Report::build(ReportKind::Error, source_name, 12)
            .with_code(3)
            .with_message(format!("{} at line {}", message, span.line + 1))
            .with_label(
                Label::new((source_name, span.start..span.end))
                    .with_message(hint)
                    .with_color(out),
            )
            .finish()
            .write((source_name, Source::from(&source)), &mut writer).unwrap();
    };

    match error {
        ExecutionError::Scan(ScanError::UnexpectedCharacter { span }) => {
            print("Unexpected character", "Remove this character", span);
        }
        ExecutionError::Scan(ScanError::UnterminatedString { span }) => {
            print(
                "Unterminated string",
                "Terminate the string with a double quote (\").",
                span,
            );
        }
        ExecutionError::Scan(ScanError::MalformedNumberLiteral { span }) => {
            print(
                "Malformed number literal",
                "Make sure that number literal is up to spec.",
                span,
            );
        }
        ExecutionError::Parse(ParseError::MissingToken { token, expected }) => {
            print(
                "Missing token",
                &format!(
                    "Add a {:?} token after \"{}\".",
                    expected,
                    token.lexeme.unwrap()
                ),
                token.span,
            );
        }
        ExecutionError::Parse(ParseError::NoTokens) => {
            print(
                "There is nothing to parse.",
                "What about adding some tokens?",
                Span::default(),
            );
        }
        ExecutionError::Parse(ParseError::InvalidAssignmentTarget { left }) => {
            print(
                "Invalid assignment target",
                &format!("No values can be assigned to {}", left.lexeme.unwrap()),
                left.span,
            );
        }
        ExecutionError::Parse(ParseError::UnexpectedToken { token }) => {
            print(
                "Unexpected token",
                &format!(
                    "Unexpected token {}.",
                    token.lexeme.unwrap_or("None".to_string())
                ),
                token.span,
            );
        }
        ExecutionError::Interpret(InterpretError::ArityMismatch {
            span,
            expected,
            found,
        }) => {
            print(
                "Function arity mismatch",
                &format!(
                    "Function expects {} arguments, while provided {}.",
                    expected, found
                ),
                span,
            );
        }
        ExecutionError::Interpret(InterpretError::UnexpectedStatement { span }) => {
            print("Unexpected statement", "Remove this.", span);
        }
        ExecutionError::Interpret(InterpretError::NotCallable { span }) => {
            print(
                "Not callable",
                "Expression does not yield a callable.",
                span,
            );
        }
        ExecutionError::Interpret(InterpretError::PropertyNotFound { property, span }) => {
            print(
                &format!(
                    "Property {} not found in the evaluated expression",
                    property
                ),
                "Check if that field is present in the expression.",
                span,
            );
        }
        ExecutionError::Plan(PlannerError::DuplicateObjectInScope { previous, ident }) => {
            print(
                "Duplicate object in scope",
                &format!("Object {} is already defined in the scope.", previous.name),
                previous.span,
            );
        }
        ExecutionError::Plan(PlannerError::SubqueryNotAllowed(span)) => {
            print(
                "Subquery not allowed",
                "Subqueries are not allowed in this context.",
                span,
            );
        }
        ExecutionError::Environment(EnvironmentError::Other { message })
        | ExecutionError::Interpret(InterpretError::Other { message }) => {
            print(&message, "", Span::default());
        }
        _ => {}
    };
}
#[cfg(test)]
mod tests {
    use super::*;
    use lykiadb_lang::tokenizer::token::{Token, TokenType};

    fn capture_error_output(filename: &str, source: &str, error: ExecutionError) -> String {
        let mut output = Vec::new();
        report_error(filename, source, error, &mut output);
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn test_scan_error_reporting() {
        let source = "let x = @";
        let error = ExecutionError::Scan(ScanError::UnexpectedCharacter {
            span: Span {
                start: 8,
                end: 9,
                line: 0,
                line_end: 0,
            },
        });

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Unexpected character"));
        assert!(output.contains("Remove this character"));
        assert!(output.contains("at line 1"));
    }

    #[test]
    fn test_interpret_error_reporting() {
        let source = "obj.nonexistent";
        let error = ExecutionError::Interpret(InterpretError::PropertyNotFound {
            property: "nonexistent".to_string(),
            span: Span {
                start: 4,
                end: 14,
                line: 0,
                line_end: 0,
            },
        });

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Property nonexistent not found"));
        assert!(output.contains("Check if that field is present"));
    }

    #[test]
    fn test_environment_error_reporting() {
        let source = "";
        let error = ExecutionError::Environment(EnvironmentError::Other {
            message: "Variable not found".to_string(),
        });

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Variable not found"));
    }
}
