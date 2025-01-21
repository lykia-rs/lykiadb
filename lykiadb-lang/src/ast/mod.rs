use std::{
    fmt::{Display, Formatter},
    hash::Hash,
    sync::Arc,
};

use derivative::Derivative;
use expr::Expr;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

pub mod expr;
pub mod sql;
pub mod stmt;
pub mod visitor;

pub trait AstNode: Spanned {
    fn get_id(&self) -> usize;
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
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
    Object(FxHashMap<String, Box<Expr>>),
    Array(Vec<Expr>),
    Undefined,
}

impl Literal {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Literal::Str(s) => Some(s),
            _ => None,
        }
    }
}

impl Hash for Literal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Literal::Str(s) => s.hash(state),
            Literal::Num(n) => n.to_bits().hash(state),
            Literal::Bool(b) => b.hash(state),
            Literal::Object(o) => (o as *const _ as usize).hash(state),
            Literal::Array(a) => a.hash(state),
            //
            Literal::Undefined => "undefined".hash(state),
        }
    }
}

impl Eq for Literal {}

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[serde(tag = "@type")]
#[derivative(Eq, PartialEq, Hash)]
pub struct Identifier {
    pub name: String,
    pub dollar: bool,
    #[serde(skip)]
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub span: Span,
}

impl Identifier {
    pub fn new(name: &str, dollar: bool) -> Self {
        Identifier {
            name: name.to_string(),
            dollar,
            span: Span::default(),
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
