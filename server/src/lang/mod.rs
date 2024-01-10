use std::rc::Rc;

use rustc_hash::FxHashMap;
use serde::Serialize;

use self::ast::expr::ExprId;

pub mod ast;
mod tests;
pub mod tokens;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Literal {
    Str(Rc<String>),
    Num(f64),
    Bool(bool),
    Undefined,
    Object(FxHashMap<String, ExprId>),
    Array(Vec<ExprId>),
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

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(tag = "@type")]
pub struct Identifier {
    pub name: String,
    pub dollar: bool,
}
