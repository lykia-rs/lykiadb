pub mod generic;
pub mod sql;
use serde_json::Value;
use assert_json_diff::assert_json_eq;
use lykiadb_server::lang::tokens::{scanner::Scanner, token::Token};

pub fn get_tokens(source: &str) -> Vec<Token> {
    return Scanner::scan(source).unwrap();
}

pub fn compare_parsed_to_expected(source: &str, expected: Value) {
    use lykiadb_server::lang::ast::parser::Parser;

    let tokens = get_tokens(source);
    let program = Parser::parse(&tokens).unwrap();
    let actual = program.to_json();
    assert_json_eq!(actual, expected);
}

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