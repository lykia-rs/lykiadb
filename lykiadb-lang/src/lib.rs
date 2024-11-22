use std::{
    fmt::{Display, Formatter, Result},
    sync::Arc,
};

use ast::expr::Expr;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

pub mod ast;
pub mod parser;
pub mod tokenizer;

pub type Scopes = Vec<FxHashMap<String, bool>>;
pub type Locals = FxHashMap<usize, usize>;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub line_end: u32,
}

pub trait Spanned {
    fn get_span(&self) -> Span;
}

impl Spanned for Span {
    fn get_span(&self) -> Span {
        *self
    }
}

impl Span {
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            line_end: self.line_end.min(other.line_end),
        }
    }
}

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
    pub span: Span,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.name)
    }
}
