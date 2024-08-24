use lykiadb_lang::ast::{expr::Expr, sql::{SqlFrom, SqlSelect}};

use crate::{engine::interpreter::HaltReason, value::types::RV};

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
        let mut node: Option<Node> = None;
        if let Some(from) = &query.core.from {
            node = Some(self.build_from(from)?);
        }
        Ok(Plan::Select(Node::Values { rows: vec![vec![]] }))
    }

    fn build_from(&mut self, from: &SqlFrom) -> Result<Node, HaltReason> {
        Err(HaltReason::Return(RV::Undefined))
    }
}
