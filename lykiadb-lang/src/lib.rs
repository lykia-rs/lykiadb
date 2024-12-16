use std::{
    fmt::{Display, Formatter},
    hash::Hash,
    sync::Arc,
};

use ast::expr::Expr;
use derivative::Derivative;
use parser::{program::Program, resolver::{ResolveError, Resolver}, ParseError, Parser};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tokenizer::scanner::{ScanError, Scanner};

pub mod ast;
pub mod parser;
pub mod tokenizer;

pub type Scopes = Vec<FxHashMap<String, bool>>;
pub type Locals = FxHashMap<usize, usize>;

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
            Literal::NaN => "NaN".hash(state),
            Literal::Null => "null".hash(state),
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


#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum LangError {
    Parse(ParseError),
    Scan(ScanError),
    Resolve(ResolveError)
}

impl From<ParseError> for LangError {
    fn from(err: ParseError) -> Self {
        LangError::Parse(err)
    }
}

impl From<ScanError> for LangError {
    fn from(err: ScanError) -> Self {
        LangError::Scan(err)
    }
}

impl From<ResolveError> for LangError {
    fn from(err: ResolveError) -> Self {
        LangError::Resolve(err)
    }
}

pub struct SourceProcessor {
    scopes: Scopes,
    locals: Locals,
}

impl Default for SourceProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceProcessor {
    pub fn new() -> SourceProcessor {
        SourceProcessor {
            scopes: vec![],
            locals: FxHashMap::default(),
        }
    }

    pub fn process(&mut self, source: &str) -> Result<Program, LangError> {
        let tokens = Scanner::scan(source)?;
        let mut program = Parser::parse(&tokens)?;
        let mut resolver = Resolver::new(self.scopes.clone(), &program, Some(self.locals.clone()));
        let (scopes, locals) = resolver.resolve()?;

        self.scopes = scopes;
        self.locals.clone_from(&locals);
        program.set_locals(self.locals.clone());

        Ok(program)
    }
}