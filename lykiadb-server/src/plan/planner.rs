use crate::{engine::{error::ExecutionError, interpreter::{HaltReason, Interpreter}}, value::RV};

use lykiadb_lang::ast::{
    expr::Expr,
    sql::{SqlFrom, SqlJoinType, SqlProjection, SqlSelect, SqlSelectCore, SqlSource},
    visitor::VisitorMut,
};

use super::{scope::Scope, Node, Plan};
pub struct Planner<'a> {
    interpreter: &'a mut Interpreter,
}

impl<'a> Planner<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Planner {
        Planner { interpreter }
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

        let mut parent_scope = Scope::new();

        // FROM/JOIN
        if let Some(from) = &core.from {
            node = self.build_from(from, &mut parent_scope)?;
        }

        // WHERE
        if let Some(predicate) = &core.r#where {
            // TODO: Traverse expression
            node = Node::Filter {
                source: Box::new(node),
                predicate: *predicate.clone(),
            }
        }

        // AGGREGATES


        // GROUP BY

        // PROJECTION
        if core.projection.as_slice() != [SqlProjection::All { collection: None }] {
            node = Node::Projection {
                source: Box::new(node),
                fields: core.projection.clone(),
            };
        }

        // HAVING

        // COMPOUND
        if let Some(compound) = &core.compound {
            node = Node::Compound {
                source: Box::new(node),
                operator: compound.operator.clone(),
                right: Box::new(self.build_select_core(&compound.core)?),
            }
        }
        Ok(node)
    }

    fn eval_constant(&mut self, expr: &Expr) -> Result<RV, HaltReason> {
        self.interpreter.visit_expr(expr)
    }

    fn build_select(&mut self, query: &SqlSelect) -> Result<Node, HaltReason> {
        let mut node: Node = self.build_select_core(&query.core)?;

        if let Some(order_by) = &query.order_by {
            node = Node::Order {
                source: Box::new(node),
                key: order_by
                    .iter()
                    .map(|x| (*x.expr.clone(), x.ordering.clone()))
                    .collect(),
            };
        }

        if let Some(limit) = &query.limit {
            if let Some(offset) = &limit.offset {
                node = Node::Offset {
                    source: Box::new(node),
                    offset: self.eval_constant(offset)?
                        .as_number()
                        .expect("Offset is not correct")
                        .floor() as usize,
                }
            }
            node = Node::Limit {
                source: Box::new(node),
                limit: self.eval_constant(&limit.count)?
                    .as_number()
                    .expect("Limit is not correct")
                    .floor() as usize,
            }
        }

        Ok(node)
    }

    fn build_from(&mut self, from: &SqlFrom, parent_scope: &mut Scope) -> Result<Node, HaltReason> {

        let mut scope = Scope::new();

        let node = match from {
            SqlFrom::Source(source) => {
                let wrapped = match source {
                    SqlSource::Collection(ident) => {
                        Node::Scan {
                            source: ident.clone(),
                            filter: None,
                        }
                    }
                    SqlSource::Expr(expr) => {
                        Node::EvalScan {
                            source: expr.clone(),
                            filter: None,
                        }
                    }
                };

                if let Err(err) = scope.add_source(source.clone()) {
                    return Err(HaltReason::Error(ExecutionError::Plan(err)));
                }

                Ok(wrapped)
            },
            SqlFrom::Select { subquery, alias } => {
                let node = Node::Subquery {
                    source: Box::new(self.build_select(subquery)?),
                    alias: alias.clone().unwrap(),
                };
                Ok(node)
            }
            SqlFrom::Group { values } => {
                let mut froms = values.iter();
                let mut node = self.build_from(froms.next().unwrap(), &mut scope)?;
                for right in froms {
                    node = Node::Join {
                        left: Box::new(node),
                        join_type: SqlJoinType::Cross,
                        right: Box::new(self.build_from(right, &mut scope)?),
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
                left: Box::new(self.build_from(left, &mut scope)?),
                join_type: join_type.clone(),
                right: Box::new(self.build_from(right, &mut scope)?),
                constraint: constraint.clone().map(|x| *x.clone()),
            }),
        };
        
        if let Err(err) = parent_scope.merge(scope){
            return Err(HaltReason::Error(ExecutionError::Plan(err)));
        }

        node
    }
}
