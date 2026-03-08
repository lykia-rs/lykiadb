mod parser;

pub use parser::{Block, ParseError, TestCase, dedent};
use parser::{flatten_items, TestLangParser};
use std::collections::HashMap;

pub trait TestHandler {
    fn run_case(&mut self, case: TestCase);
}

pub struct TestRunner {
    handler_fn: Box<dyn Fn() -> Box<dyn TestHandler>>,
}

impl TestRunner {
    pub fn new(handler_fn: Box<dyn Fn() -> Box<dyn TestHandler>>) -> TestRunner {
        TestRunner { handler_fn }
    }

    pub fn test_file(&mut self, input: &str) {
        let items = TestLangParser::new(input)
            .parse()
            .expect("Failed to parse test file");

        let cases = flatten_items(&items, &HashMap::new());

        for case in cases {
            let mut handler = (self.handler_fn)();
            handler.run_case(case);
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
