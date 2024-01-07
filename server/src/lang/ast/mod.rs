use serde::Serialize;
use std::rc::Rc;
use rustc_hash::FxHashMap;
use self::expr::ExprId;
pub mod expr;
pub mod parser;
pub mod program;
pub mod sql;
pub mod stmt;
pub mod visitor;

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
