extern crate wasm_bindgen;
extern crate lykiadb_lang;

use lykiadb_lang::{parser::{resolver::Resolver, Parser}, tokenizer::scanner::Scanner, Scopes};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(source: &str) -> Result<JsValue, JsValue> {

    let tokens = Scanner::scan(source).unwrap();
    let parse_result = Parser::parse(&tokens);

    if let Ok(mut program) = parse_result {
        let mut resolver = Resolver::new(Scopes::default(), &program, None);
        let (scopes, locals) = resolver.resolve().unwrap();
        program.set_locals(locals);
        return Ok(serde_wasm_bindgen::to_value(&program)?);
    }

    Ok(serde_wasm_bindgen::to_value(&parse_result.err())?)
}