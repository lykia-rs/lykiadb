use crate::engine::interpreter::{ExecutionContext, HaltReason};
use lykiadb_lang::ast::{
    expr::Expr,
    sql::{SqlFrom, SqlJoinType, SqlSelect, SqlSelectCore}
};

use super::{Node, Plan};
pub struct Planner {
    context: ExecutionContext
}

impl Planner {
    pub fn new(ctx: ExecutionContext) -> Planner {
        Planner {
            context: ctx
        }
    }

    pub fn build(&mut self, expr: &Expr) -> Result<Plan, HaltReason> {
        match expr {
            Expr::Select {
                query,
                span: _,
                id: _,
            } => {
                let plan = Plan::Select(self.build_select(query)?);
                Ok(plan)
            }
            _ => panic!("Not implemented yet."),
        }
    }

    fn build_select_core(&mut self, core: &SqlSelectCore) -> Result<Node, HaltReason> {
        let mut node: Node = Node::Nothing;
        // FROM/JOIN
        if let Some(from) = &core.from {
            node = self.build_from(from)?;
        }
        // WHERE
        if let Some(predicate) = &core.r#where {
            // TODO: Traverse expression
            node = Node::Filter {
                source: Box::new(node),
                predicate: *predicate.clone(),
            }
        }
        if let Some(compound) = &core.compound {
            node = Node::Compound { 
                source: Box::new(node),
                operator: compound.operator.clone(),
                right: Box::new(self.build_select_core(&compound.core)?)
            }
        }
        // GROUP BY
        // HAVING
        // SELECT
        Ok(node)
    }

    fn build_select(&mut self, query: &SqlSelect) -> Result<Node, HaltReason> {
        let mut node: Node = self.build_select_core(&query.core)?;

        // ORDER BY

        if let Some(limit) = &query.limit {
            if let Some(offset) = &limit.offset {
                node = Node::Offset { source: Box::new(node), offset: self.context.eval(&offset)?.as_number().expect("Offset is not correct").floor() as usize }
            }

            node = Node::Limit { source: Box::new(node), limit: self.context.eval(&limit.count)?.as_number().expect("Limit is not correct").floor() as usize }
        }

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


pub mod test_helpers {
    use lykiadb_lang::{ast::stmt::Stmt, parser::program::Program};

    use crate::engine::interpreter::ExecutionContext;

    use super::Planner;

    pub fn expect_plan(query: &str, expected_plan: &str) {
        let ctx = ExecutionContext::new(None);
        let mut planner = Planner::new(ctx);
        let program = query.parse::<Program>().unwrap();
        match *program.get_root() {
            Stmt::Program { body, .. } if matches!(body.get(0), Some(Stmt::Expression { .. })) => {
                if let Some(Stmt::Expression { expr, .. }) = body.get(0) {
                    let generated_plan = planner.build(&expr).unwrap();
                    assert_eq!(expected_plan, generated_plan.to_string());
                }
            }
            _ => panic!("Expected expression statement."),
        }
    }
}
