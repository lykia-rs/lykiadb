use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub line_end: u32,
}

pub trait StandardError {
    fn get_message(&self) -> String;
    fn get_hint(&self) -> String;
    fn get_error_code(&self) -> String;
    fn get_span(&self) -> Option<Span>;
}

pub fn report_error(
    source_name: &str,
    source: &str,
    error: Box<dyn StandardError>,
    mut writer: impl std::io::Write,
) {
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
            .write((source_name, Source::from(&source)), &mut writer)
            .unwrap();
    };

    print(
        &format!(
            "[{}] {}",
            &error.get_error_code().as_str(),
            &error.get_message().as_str()
        ),
        error.get_hint().as_str(),
        error.get_span().unwrap(),
    );
}