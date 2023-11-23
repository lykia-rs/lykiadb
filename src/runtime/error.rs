use crate::lang::{parser::ParseError, scanner::ScanError, token::Span};

use super::resolver::ResolveError;

pub enum ExecutionError {
    Scan(ScanError),
    Parse(ParseError),
    Resolve(ResolveError),
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
            print(&"Unexpected character", &"Remove this character", span);
        }
        ExecutionError::Scan(ScanError::UnterminatedString { span }) => {
            print(
                &"Unterminated string",
                &"Terminate the string with a double quote (\").",
                span,
            );
        }
        ExecutionError::Scan(ScanError::MalformedNumberLiteral { span }) => {
            print(
                &"Malformed number literal",
                &"Make sure that number literal is up to spec.",
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
        _ => {}
    }
}
