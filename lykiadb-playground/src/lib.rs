extern crate wasm_bindgen;
extern crate lykiadb_lang;
extern crate rustc_hash;
extern crate serde;

use lykiadb_lang::{parser::{program::Program, resolver::Resolver, Parser}, tokenizer::scanner::Scanner, Scopes};
use wasm_bindgen::prelude::*;
mod regularizer;

#[wasm_bindgen]
pub fn parse(source: &str) -> Result<JsValue, JsValue> {

    let tokens = Scanner::scan(source).unwrap();
    let mut parser = Parser::create(&tokens);
    let parse_result = parser.program();


    if let Ok(mut r) = parse_result {
        let mut program = Program::new(r);
        let mut resolver = Resolver::new(Scopes::default(), &program, None);
        let (scopes, locals) = resolver.resolve().unwrap();
        program.set_locals(locals);
        let tree = regularizer::TreeBuilder::new(parser.get_metadata()).build(&program.get_root());
        return Ok(serde_wasm_bindgen::to_value(&tree)?);
    }

    Ok(serde_wasm_bindgen::to_value(&parse_result.err())?)
}