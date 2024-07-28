use std::sync::Arc;

use ast::expr::Expr;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tokenizer::token::Span;

pub mod ast;
pub mod parser;
pub mod tokenizer;

pub type Scopes = Vec<FxHashMap<String, bool>>;
pub type Locals = FxHashMap<usize, usize>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Str(Arc<String>),
    Num(f64),
    Bool(bool),
    Undefined,
    Object(FxHashMap<String, Box<Expr>>),
    Array(Vec<Expr>),
    NaN,
    Null,
}

impl Literal {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Literal::Str(s) => Some(s),
            _ => None,
        }
    }
}

impl Eq for Literal {}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub struct Identifier {
    pub name: String,
    pub dollar: bool,
    #[serde(skip)]
    pub span: Span
}
