use std::marker::PhantomData;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};

use self::{expr::Expr, stmt::Stmt};

pub mod expr;
pub mod sql;
pub mod stmt;
pub mod visitor;

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Deserialize)]
pub struct AstRef<T>(pub usize, PhantomData<T>);

impl<T> AstRef<T> {
    pub fn new(num: u32) -> Self {
        AstRef(num as usize, PhantomData)
    }
}

impl<T> Clone for AstRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for AstRef<T> {}

pub struct AstPool<T> {
    nodes: Vec<T>,
}

impl<T> AstPool<T> {
    pub fn new() -> Self {
        AstPool { nodes: vec![] }
    }

    pub fn alloc(&mut self, expr: T) -> AstRef<T> {
        self.nodes.push(expr);
        AstRef::<T>(self.nodes.len() - 1, std::marker::PhantomData)
    }

    pub fn get(&self, idx: AstRef<T>) -> &T {
        &self.nodes[idx.0]
    }
}

pub struct AstArena {
    pub expressions: AstPool<Expr>,
    pub statements: AstPool<Stmt>,
}

impl AstArena {
    pub fn new() -> AstArena {
        AstArena {
            expressions: AstPool::new(),
            statements: AstPool::new(),
        }
    }
}

pub const EXPR_ID_PLACEHOLDER: &'static str = "@ExprId";
pub const STMT_ID_PLACEHOLDER: &'static str = "@StmtId";

impl Serialize for AstRef<Expr> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(EXPR_ID_PLACEHOLDER, &self.0)?;
        map.end()
    }
}

impl Serialize for AstRef<Stmt> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(STMT_ID_PLACEHOLDER, &self.0)?;
        map.end()
    }
}
