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
    let program = Parser::parse(&tokens).unwrap();
    let actual = program.to_json();
    assert_json_eq!(actual, expected);
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_parsing {
    ($($name:ident: {$field:literal => $value:tt}),*) => {
        $(
            #[test]
            fn $name() {
                compare_parsed_to_expected($field, json!($value));
            }
        )*
    };
}
