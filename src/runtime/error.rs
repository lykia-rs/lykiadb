use crate::lang::{parser::ParseError, scanner::ScanError};

use super::resolver::ResolveError;

pub enum ExecutionError {
    Scan(ScanError),
    Parse(ParseError),
    Resolve(ResolveError),
}
/*
fn report() {
    use ariadne::{Color, ColorGenerator, Fmt, Label, Report, ReportKind, Source};

    let mut colors = ColorGenerator::new();

    // Generate & choose some colours for each of our elements
    let a = colors.next();
    let b = colors.next();
    let out = Color::Fixed(81);

    Report::build(ReportKind::Error, filename, 12)
        .with_code(3)
        .with_message(format!("Incompatible types"))
        .with_label(
            Label::new((filename, 12..13))
                .with_message(format!("This is of type {}", "Nat".fg(a)))
                .with_color(a),
        )
        .with_label(
            Label::new((filename, 2..5))
                .with_message(format!("This is of type {}", "Str".fg(b)))
                .with_color(b),
        )
        .with_label(
            Label::new((filename, 1..4))
                .with_message(format!(
                    "The values are outputs of this {} expression",
                    "match".fg(out),
                ))
                .with_color(out),
        )
        .with_note(format!(
            "Outputs of {} expressions must coerce to the same type",
            "match".fg(out)
        ))
        .finish()
        .print((filename, Source::from(filename)))
        .unwrap();
} */
