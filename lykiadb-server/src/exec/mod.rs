use interb::Symbol;
use lykiadb_lang::ast::sql::SqlProjection;
use rustc_hash::FxHashMap;
use serde_json::map::Iter;

use crate::{engine::{error::ExecutionError, interpreter::{HaltReason, Interpreter}}, global::GLOBAL_INTERNER, plan::{Node, Plan}, value::{RV, iterator::{IterationEnvironment, RVIterator, RVs}, object::RVObject}};

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
            Node::Projection { source, fields } => {
                let cursor = self.execute_node(*source)?;

                let mut inter_fork = self.interpreter.clone();

                let iter = cursor.map(move |downstream: IterationEnvironment| {
                    let mut upstream = IterationEnvironment::new(vec![], vec![]);

                    for field in &fields {
                        match field {
                            SqlProjection::All { collection } => {
                                if collection.is_none() {
                                    // env.spread_to(&mut row);
                                } else {
                                    let projected_key = collection.as_ref().unwrap().to_string();
                                    let interned_key = GLOBAL_INTERNER.intern(&projected_key);
                                    let value = &downstream.get(&interned_key);
                                    // row.insert(interned_key, value.unwrap().clone());
                                }
                            },
                            SqlProjection::Expr { expr, alias } => {
                                let evaluated = inter_fork.eval_with_iter(&expr, &downstream);
                                let value = match evaluated {
                                    Ok(v) => v,
                                    Err(_) => RV::Undefined,
                                };
                                let key = alias.as_ref().unwrap().to_string();
                                upstream.insert(
                                    GLOBAL_INTERNER.intern(&key),
                                    value,
                                );
                            }
                        }
                    }

                    upstream
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

                let sym_alias = GLOBAL_INTERNER.intern(&alias.to_string());
                let mapper = move |v: RV| IterationEnvironment::new(
                    vec![sym_alias.clone()],
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
