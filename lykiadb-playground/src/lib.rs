extern crate lykiadb_lang;
extern crate rustc_hash;
extern crate serde;
extern crate wasm_bindgen;

use lykiadb_lang::{
    parser::{resolver::Resolver, Parser},
    tokenizer::scanner::Scanner,
    Scopes,
};
use regularizer::{Tree, TreeBuilder};
use wasm_bindgen::prelude::*;
mod regularizer;

#[wasm_bindgen]
pub fn parse(source: &str) -> Result<JsValue, JsValue> {
    let tokens = Scanner::scan(source).unwrap();
    let parse_result = Parser::parse(&tokens);

    if let Ok(mut program) = parse_result {
        let mut resolver = Resolver::new(Scopes::default(), &program, None);
        let (_, locals) = resolver.resolve().unwrap();
        program.set_locals(locals);
        return Ok(serde_wasm_bindgen::to_value(&program)?);
    }

    Ok(serde_wasm_bindgen::to_value(&parse_result.err())?)
}

#[wasm_bindgen]
pub fn tokenize(source: &str) -> Result<JsValue, JsValue> {
    let tokens = Scanner::scan(source).unwrap();

    let mut last = tokens.last().unwrap().span;

    last.start = 0;
    last.line = 0;

    let token_tree = Tree {
        name: "Program".to_owned(),
        children: Some(tokens.into_iter().map(TreeBuilder::token_to_tree).collect()),
        span: last,
    };

    Ok(serde_wasm_bindgen::to_value(&token_tree)?)
}
