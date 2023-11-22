use crate::lang::{parser::ParseError, scanner::ScanError};

use super::resolver::ResolveError;

pub enum ExecutionError {
    Scan(ScanError),
    Parse(ParseError),
    Resolve(ResolveError),
}

pub fn report_error(filename: &str, source: &str, error: ScanError) {
    use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

    // Generate & choose some colours for each of our elements
    let out = Color::Fixed(81);

    match error {
        ScanError::UnexpectedCharacter { span, message }|
        ScanError::UnterminatedString { span, message } |
        ScanError::MalformedNumberLiteral { span, message } => {
            Report::build(ReportKind::Error, filename, 12)
            .with_code(3)
            .with_message(message)
            .with_label(
                Label::new((filename, span.start..span.start + span.lexeme.len()))
                    .with_message(format!(
                        "The values are outputs of this {} expression",
                        "match".fg(out),
                    ))
                    .with_color(out),
            )
            .finish()
            .print((filename, Source::from(&source)))
            .unwrap();
        }
    }
}