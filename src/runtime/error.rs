use std::rc::Rc;

use crate::lang::{parser::ParseError, scanner::ScanError, token::Span};

use super::{interpreter::InterpretError, resolver::ResolveError};

#[derive(Debug, Clone)]
pub enum ExecutionError {
    Scan(ScanError),
    Parse(ParseError),
    Resolve(ResolveError),
    Interpret(InterpretError),
}

pub fn report_error(filename: &str, source: &str, error: ExecutionError) {
    use ariadne::{Color, Label, Report, ReportKind, Source};

    // Generate & choose some colours for each of our elements
    let out = Color::Fixed(81);

    let print = |message: &str, hint: &str, span: Span| {
        Report::build(ReportKind::Error, filename, 12)
            .with_code(3)
            .with_message(format!("{} at line {}", message, span.line + 1))
            .with_label(
                Label::new((filename, span.start..(span.start + span.lexeme.len())))
                    .with_message(hint)
                    .with_color(out),
            )
            .finish()
            .print((filename, Source::from(&source)))
            .unwrap();
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
                    expected, token.span.lexeme
                ),
                token.span,
            );
        }
        ExecutionError::Parse(ParseError::InvalidAssignmentTarget { left }) => {
            print(
                "Invalid assignment target",
                &format!("No values can be assigned to {}", left.span.lexeme),
                left.span,
            );
        }
        ExecutionError::Parse(ParseError::UnexpectedToken { token }) => {
            print(
                "Unexpected token",
                &format!("Unexpected token {}.", token.span.lexeme),
                token.span,
            );
        }
        ExecutionError::Interpret(InterpretError::ArityMismatch {
            token,
            expected,
            found,
        }) => {
            print(
                "Function arity mismatch",
                &format!(
                    "Function expects {} arguments, while provided {}.",
                    expected, found
                ),
                token.span,
            );
        }
        ExecutionError::Interpret(InterpretError::UnexpectedStatement { token }) => {
            print(
                "Unexpected statement",
                &format!("Unexpected \"{}\" statement.", token.span.lexeme,),
                token.span,
            );
        }
        ExecutionError::Interpret(InterpretError::NotCallable { token }) => {
            print(
                "Not callable",
                &format!(
                    "Expression does not yield a callable {}.",
                    token.span.lexeme,
                ),
                token.span,
            );
        }
        /*ExecutionError::Interpret(InterpretError::AssignmentToUndefined { token }) => {
            print(
                "Assignment to an undefined variable",
                &format!(
                    "{} is undefined, so no value can be assigned to it.",
                    token.span.lexeme,
                ),
                token.span,
            );
        }
        ExecutionError::Interpret(InterpretError::VariableNotFound { token }) => {
            print(
                "Variable not found",
                &format!("{} is not defined, cannot be evaluated.", token.span.lexeme,),
                token.span,
            );
        }*/
        ExecutionError::Interpret(InterpretError::Other { message }) => {
            print(
                &message,
                "",
                Span {
                    start: 0,
                    lexeme: Rc::new("".to_string()),
                    line: 0,
                },
            );
        }
        _ => {}
    }
}
