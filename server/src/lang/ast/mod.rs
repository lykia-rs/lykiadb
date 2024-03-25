use serde::Deserialize;
use std::marker::PhantomData;

use self::{expr::Expr, sql::SqlExpr, stmt::Stmt};

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
    pub sql_expressions: AstPool<SqlExpr>,
    pub statements: AstPool<Stmt>,
}

impl AstArena {
    pub fn new() -> AstArena {
        AstArena {
            expressions: AstPool::new(),
            sql_expressions: AstPool::new(),
            statements: AstPool::new(),
        }
    }
}
