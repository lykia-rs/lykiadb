use lykiadb_lang::ast::sql::SqlProjection;

use crate::{
    engine::{
        error::ExecutionError,
        interpreter::{HaltReason, Interpreter},
    },
    global::GLOBAL_INTERNER,
    plan::{Node, Plan},
    value::{
        RV,
        iterator::{ExecutionRow, RVs},
    },
};

pub struct PlanExecutor<'a> {
    interpreter: &'a mut Interpreter,
}

impl<'a> PlanExecutor<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> PlanExecutor<'a> {
        PlanExecutor { interpreter }
    }

    pub fn execute_plan(&mut self, plan: Plan) -> Result<RVs, ExecutionError> {
        // Placeholder for plan execution logic
        match plan {
            Plan::Select(root) => {
                // Execute scan plan
                self.execute_node(root.clone())
            }
        }
    }

    pub fn execute_node(&mut self, node: Node) -> Result<RVs, ExecutionError> {
        match node {
            Node::Subquery { source, alias } => {
                let cursor = self.execute_node(*source)?;

                let iter = cursor.map(move |row: ExecutionRow| {
                    let mut upstream = ExecutionRow::new();
                    let key = alias.to_string();
                    upstream.insert(GLOBAL_INTERNER.intern(&key), row.as_value());
                    upstream
                });

                Ok(Box::from(iter))
            }
            Node::Offset { source, offset } => {
                Ok(Box::from(self.execute_node(*source)?.skip(offset)))
            }
            Node::Limit { source, limit } => {
                Ok(Box::from(self.execute_node(*source)?.take(limit)))
            }
            Node::Filter { source, predicate, subqueries }
            => {
                if predicate.is_constant() {
                    // TODO(vck): Maybe we can deal with this at compile time?
                    let constant_evaluation = predicate.as_bool().map(|b| {
                        if b {
                            let cursor = self.execute_node(*source)?;
                            Ok(cursor)
                        } else {
                            let empty_iter = Vec::<ExecutionRow>::new().into_iter();
                            Ok(Box::from(empty_iter) as RVs)
                        }
                    });

                    return constant_evaluation.unwrap();
                }

                let cursor = self.execute_node(*source)?;

                let mut inter_fork = self.interpreter.clone();

                let expr = predicate.as_expr().unwrap().clone();

                let iter = cursor.filter_map(move |row: ExecutionRow| {
                    let evaluated = inter_fork.eval_with_row(&expr, &row);
                    if let Ok(value) = evaluated && value.as_bool() {
                        Some(row)
                    } else {
                        None
                    }
                });

                Ok(Box::from(iter))
            }
            Node::Projection { source, fields } => {
                let cursor = self.execute_node(*source)?;

                let mut inter_fork = self.interpreter.clone();

                let iter = cursor.map(move |downstream: ExecutionRow| {
                    let mut upstream = ExecutionRow::new();

                    for field in &fields {
                        match field {
                            SqlProjection::All { collection } => {
                                if collection.is_none() {
                                    downstream.copy_to(&mut upstream);
                                } else {
                                    let projected_key = collection.as_ref().unwrap().to_string();
                                    let interned_key = GLOBAL_INTERNER.intern(&projected_key);
                                    let value = &downstream.get(&interned_key);
                                    upstream.insert(interned_key, value.unwrap().clone());
                                }
                            }
                            SqlProjection::Expr { expr, alias } => {
                                let evaluated = inter_fork.eval_with_row(&expr, &downstream);
                                let value = match evaluated {
                                    Ok(v) => v,
                                    Err(_) => RV::Undefined,
                                };
                                let key = alias.as_ref().unwrap().to_string();
                                upstream.insert(GLOBAL_INTERNER.intern(&key), value);
                            }
                        }
                    }

                    upstream
                });

                Ok(Box::from(iter))
            }
            Node::EvalScan { source, filter } => {
                let mut evaluated = self.interpreter.eval(&source.expr);
                if let Err(e) = evaluated {
                    match e {
                        HaltReason::Error(err) => return Err(err),
                        HaltReason::Return(value) => {
                            evaluated = Ok(value);
                        }
                    }
                }

                let alias = source.alias.to_owned();
                let value = evaluated.unwrap();

                let sym_alias = GLOBAL_INTERNER.intern(&alias.to_string());

                let mapper = move |v: RV| {
                    let mut env = ExecutionRow::new();
                    env.insert(sym_alias.clone(), v.clone());
                    env
                };

                let iter = match value {
                    RV::Array(arr) => {
                        let c = arr.collect();
                        c.into_iter().map(mapper)
                    }
                    _ => vec![value]
                        .into_iter()
                        .collect::<Vec<_>>()
                        .into_iter()
                        .map(mapper),
                };

                Ok(Box::from(iter))
            }
            _ => panic!("Unsupported node type"),
        }
    }
}
