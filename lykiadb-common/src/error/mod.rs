use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub line_end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardError {
    pub message: String,
    pub hint: String,
    pub error_code: String,
    pub span: Option<Span>,
}

impl StandardError {
    pub fn new(message: &str, hint: &str, span: Option<Span>) -> Self {
        StandardError {
            message: message.to_owned(),
            hint: hint.to_owned(),
            error_code: "000".to_owned(),
            span,
        }
    }

    pub fn report(&self, source_name: &str, source: &str, mut writer: impl std::io::Write) {
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
            &format!("[{}] {}", &self.error_code.as_str(), &self.message.as_str()),
            self.hint.as_str(),
            self.span.unwrap(),
        );
    }
}
