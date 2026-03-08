mod parser;

pub use parser::{Block, ParseError, TestCase, dedent};
use parser::{TestLangParser, flatten_items};
use std::collections::HashMap;
use std::fmt;
use std::io::Write;

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

fn file_prefix(filename: &str) -> String {
    use std::path::{Component, Path};
    let path = Path::new(filename);
    let components: Vec<Component> = path.components().collect();
    let rel_start = components
        .iter()
        .position(|c| c.as_os_str() == "tests")
        .map(|i| i + 1)
        .unwrap_or(0);
    let rel: std::path::PathBuf = components[rel_start..].iter().collect();
    rel.with_extension("").to_string_lossy().into_owned()
}

#[derive(Debug)]
pub struct TestFailure(pub String);

impl fmt::Display for TestFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestFailure {}

pub trait TestHandler {
    fn run_case(&mut self, case: TestCase) -> Result<(), TestFailure>;
}

pub struct TestRunner {
    handler_fn: Box<dyn Fn() -> Box<dyn TestHandler>>,
}

impl TestRunner {
    pub fn new(handler_fn: Box<dyn Fn() -> Box<dyn TestHandler>>) -> TestRunner {
        TestRunner { handler_fn }
    }

    pub fn test_file(&mut self, input: &str) {
        self.test_file_named("", input);
    }

    pub fn test_file_named(&mut self, filename: &str, input: &str) {
        let items = TestLangParser::new(input)
            .parse()
            .expect("Failed to parse test file");

        let cases = flatten_items(&items, &HashMap::new(), "");

        let prefix = if filename.is_empty() {
            String::new()
        } else {
            file_prefix(filename)
        };

        let mut buf = String::new();
        let mut passed = 0usize;
        let mut failed = 0usize;

        for case in cases {
            let file_tag = if prefix.is_empty() {
                String::new()
            } else {
                format!(" {DIM}({prefix}){RESET}")
            };
            let test_name = case.name.clone();
            let mut handler = (self.handler_fn)();
            match handler.run_case(case) {
                Ok(()) => {
                    buf.push_str(&format!(
                        "[{BOLD}{GREEN}PASS{RESET}] {test_name}{file_tag}\n"
                    ));
                    passed += 1;
                }
                Err(TestFailure(diff)) => {
                    buf.push_str(&format!(
                        "[{BOLD}{RED}FAIL{RESET}] {test_name}{file_tag}\n{diff}\n"
                    ));
                    failed += 1;
                }
            }
        }

        // Write the entire file's output atomically so parallel tests don't interleave.
        {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            let _ = write!(handle, "{buf}");
            let _ = handle.flush();
        }

        if failed > 0 {
            panic!("{failed} test(s) failed, {passed} passed");
        }
    }
}

#[macro_export]
macro_rules! extract {
    ($pat: pat, $expr:expr) => {
        let $pat = $expr else {
            panic!("Extract pattern did not match");
        };
    };
}
