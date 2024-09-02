use crate::engine::interpreter::HaltReason;
use lykiadb_lang::ast::{
    expr::Expr,
    sql::{SqlFrom, SqlJoinType, SqlSelect},
};

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
            } => {
                let plan = Plan::Select(self.build_select(query)?);
                println!("{}", serde_json::to_value(&plan).unwrap());
                Ok(plan)
            }
            _ => panic!("Not implemented yet."),
        }
    }

    fn build_select(&mut self, query: &SqlSelect) -> Result<Node, HaltReason> {
        let mut node: Node = Node::Nothing;
        // FROM/JOIN
        if let Some(from) = &query.core.from {
            node = self.build_from(from)?;
        }
        // WHERE
        if let Some(where_clause) = &query.core.r#where {
            // TODO: Traverse expression
            node = Node::Filter {
                source: Box::new(node),
                predicate: *where_clause.clone(),
            }
        }
        // GROUP BY
        // HAVING
        // SELECT
        // ORDER BY
        // LIMIT/OFFSET
        Ok(node)
    }

    fn build_from(&mut self, from: &SqlFrom) -> Result<Node, HaltReason> {
        match from {
            SqlFrom::Collection(ident) => Ok(Node::Scan {
                source: ident.clone(),
                filter: None,
            }),
            SqlFrom::Select { subquery, alias } => {
                let node = Node::Subquery {
                    source: Box::new(self.build_select(subquery)?),
                    alias: alias.clone().unwrap(),
                };
                Ok(node)
            }
            SqlFrom::Group { values } => {
                let mut froms = values.iter();
                let mut node = self.build_from(froms.next().unwrap())?;
                for right in froms {
                    node = Node::Join {
                        left: Box::new(node),
                        join_type: SqlJoinType::Cross,
                        right: Box::new(self.build_from(right)?),
                        constraint: None,
                    }
                }
                Ok(node)
            }
            SqlFrom::Join {
                left,
                join_type,
                right,
                constraint,
            } => Ok(Node::Join {
                left: Box::new(self.build_from(left)?),
                join_type: join_type.clone(),
                right: Box::new(self.build_from(right)?),
                constraint: constraint.clone().map(|x| *x.clone()),
            }),
        }
    }
}
