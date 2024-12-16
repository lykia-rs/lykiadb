use parser::{
    program::Program,
    resolver::{ResolveError, Resolver},
    ParseError, Parser,
};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tokenizer::scanner::{ScanError, Scanner};

pub mod ast;
pub mod parser;
pub mod tokenizer;

pub type Scopes = Vec<FxHashMap<String, bool>>;
pub type Locals = FxHashMap<usize, usize>;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum LangError {
    Parse(ParseError),
    Scan(ScanError),
    Resolve(ResolveError),
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
