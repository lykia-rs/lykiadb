#[cfg(test)]
use assert_json_diff::assert_json_eq;

#[cfg(test)]
use serde_json::Value;

#[cfg(test)]
use crate::lang::{parser::Parser, scanner::Scanner, token::Token};

#[macro_export]
macro_rules! lexm {
    ($a: literal) => {
        Some($a.to_owned())
    };
}

#[cfg(test)]
pub fn get_tokens(source: &str) -> Vec<Token> {
    return Scanner::scan(source).unwrap();
}

#[cfg(test)]
pub fn compare_parsed_to_expected(source: &str, expected: Value) {
    let tokens = get_tokens(source);
    let mut parsed = Parser::parse(&tokens).unwrap();
    let actual = parsed.to_json();
    assert_json_eq!(actual, expected);
}

#[macro_export]
#[cfg(test)]
macro_rules! generate_test_cases {
    ($dir:expr, $($file:ident),*) => {
        $(
            #[test]
            fn $file() {
                let path = format!("src/lang/tests/{}/{}.json", $dir, stringify!($file));
                let content_json = fs::read_to_string(&path).unwrap();

                let content: Value = from_str(&content_json).unwrap();
                let source = content["source"].as_str().unwrap();
                let expected = content["expected"].clone();
                compare_parsed_to_expected(source, expected);
            }
        )*
    };
}
