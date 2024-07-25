extern crate wasm_bindgen;
extern crate lykiadb_lang;
extern crate serde_json;

use serde_json::json;
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

        let result = json!({
            "scopes": &scopes,
            "program": &program,
            "tokens": &tokens
        }); 
        return Ok(serde_wasm_bindgen::to_value(&result)?);
    }

    let result = json!({
        "tokens": &tokens
    });
    Ok(serde_wasm_bindgen::to_value(&result)?)
}