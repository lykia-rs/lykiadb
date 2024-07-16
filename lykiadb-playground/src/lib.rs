extern crate wasm_bindgen;
extern crate lykiadb_lang;

use lykiadb_lang::{parser::{resolver::Resolver, Parser}, tokenizer::scanner::Scanner, Scopes};
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub fn parse(source: &str) -> i32 {

    let tokens = Scanner::scan(source).unwrap();
    let mut program = Parser::parse(&tokens).unwrap();
    let mut resolver = Resolver::new(Scopes::default(), &program, None);
    let (scopes, locals) = resolver.resolve().unwrap();
    program.set_locals(locals);

    32
}