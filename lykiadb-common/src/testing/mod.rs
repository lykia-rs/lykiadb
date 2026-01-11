pub trait TestHandler {
    fn run_case(&mut self, case_parts: Vec<String>, flags: std::collections::HashMap<&str, &str>);
}

pub struct TestRunner {
    handler_fn: Box<dyn Fn() -> Box<dyn TestHandler>>,
}

impl TestRunner {
    pub fn new(handler_fn: Box<dyn Fn() -> Box<dyn TestHandler>>) -> TestRunner {
        TestRunner { handler_fn }
    }

    pub fn test_file(&mut self, input: &str) {
        let parts: Vec<&str> = input.split("#[").collect();

        for part in parts[1..].iter() {
            let mut handler = (self.handler_fn)();

            let directives_and_input = part.trim();

            let directives_end = directives_and_input
                .find('>')
                .unwrap_or(directives_and_input.len());

            let rest = directives_and_input[directives_end + 1..]
                .trim()
                .to_string();

            let flags = directives_and_input[..directives_end - 1]
                .trim()
                .split(',')
                .map(|flag| {
                    let kv: Vec<&str> = flag.split('=').collect();
                    (kv[0].trim(), kv[1].trim())
                })
                .fold(std::collections::HashMap::new(), |mut acc, (k, v)| {
                    acc.insert(k, v);
                    acc
                });

            let case_parts = rest.split("---").map(|x| x.trim().to_string()).collect();

            match flags.get("run") {
                Some(&"plan") | Some(&"interpreter") => {
                    handler.run_case(case_parts, flags.clone());
                }
                _ => panic!("Unknown directive"),
            }
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
