use lykiadb_lang::ast::{expr::Expr, sql::SqlSelect};

use crate::{engine::interpreter::HaltReason};

use super::{Node, Plan};
pub struct Planner;

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}

impl Planner {
    pub fn new() -> Planner {
        Planner {}
    }

    pub fn build(&mut self, expr: &Expr) -> Result<Plan, HaltReason> {
        match expr {
            Expr::Select {
                query,
                span: _,
                id: _,
            } => self.build_select(query),
            _ => panic!("Not implemented yet."),
        }
    }

    fn build_select(&mut self, query: &SqlSelect) -> Result<Plan, HaltReason> {
        Ok(Plan::Select(Node::Values { rows: vec![vec![]] }))
    }
}
