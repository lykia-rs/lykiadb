use lykiadb_lang::ast::sql::SqlProjection;
use rustc_hash::FxHashMap;

use crate::{engine::{error::ExecutionError, interpreter::{HaltReason, Interpreter}}, plan::{Node, Plan}, value::{RV, iterator::{IterationEnvironment, RVIterator, RVs}, object::RVObject}};

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
        // Placeholder for node execution logic
        match node {
            Node::Projection { source, fields } => {
                let cursor = self.execute_node(*source)?;

                let iter = cursor.map(move |env: IterationEnvironment| {
                    let mut row: FxHashMap<String, RV> = FxHashMap::default();

                    for field in &fields {
                        match field {
                            SqlProjection::All { collection } => {
                                if collection.is_none() {
                                    // Select all fields from the environment
                                    &env.spread_to(&mut row);
                                } else {
                                    let projected_key = collection.as_ref().unwrap().to_string();
                                    let value = &env.get(&projected_key);
                                    row.insert(projected_key, value.unwrap().clone());
                                }
                            },
                            SqlProjection::Expr { expr, alias } => {
                                self.interpreter.set_iteration_environment(Some(env.clone()));
                                let evaluated = self.interpreter.eval(&expr);
                                self.interpreter.clear_iteration_environment();
                                let value = match evaluated {
                                    Ok(v) => v,
                                    Err(_) => RV::Undefined,
                                };
                                let key = alias.as_ref().unwrap().to_string();
                                row.insert(key, value);
                            }
                        }
                    }
                    IterationEnvironment::new(vec!["0".to_owned()], vec![RV::Object(RVObject::from_map(row))])
                });

                Ok(Box::from(iter))
            },
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

                let mapper = move |v: RV| IterationEnvironment::new(
                    vec![alias.to_string()],
                    vec![v.clone()]
                );

                let iter = match value {
                    RV::Array(arr) => {
                        let c = arr.collect();
                        c.into_iter().map(mapper)
                    },
                    _ => {
                        vec![value].into_iter().collect::<Vec<_>>().into_iter().map(mapper)
                    }
                };

                Ok(Box::from(iter))
            },
            _ => panic!("Unsupported node type"),
        }
    }
}
