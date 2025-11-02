use interb::Symbol;
use lykiadb_lang::ast::sql::SqlProjection;
use rustc_hash::FxHashMap;

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

                let iter = cursor.map(move |env: IterationEnvironment| {
                    let mut row: FxHashMap<Symbol, RV> = FxHashMap::default();

                    for field in &fields {
                        match field {
                            SqlProjection::All { collection } => {
                                if collection.is_none() {
                                    env.spread_to(&mut row);
                                } else {
                                    let projected_key = collection.as_ref().unwrap().to_string();
                                    let interned_key = GLOBAL_INTERNER.intern(&projected_key);
                                    let value = &env.get(&interned_key);
                                    row.insert(interned_key, value.unwrap().clone());
                                }
                            },
                            SqlProjection::Expr { expr, alias } => {
                                let evaluated = inter_fork.eval_with_iter(&expr, &env);
                                let value = match evaluated {
                                    Ok(v) => v,
                                    Err(_) => RV::Undefined,
                                };
                                let key = alias.as_ref().unwrap().to_string();
                                row.insert(GLOBAL_INTERNER.intern(&key), value);
                            }
                        }
                    }

                    // TODO(vck): The following is terrible, fix it.
                    let pairs = row.iter().map(|(k, v)| (GLOBAL_INTERNER.resolve(*k).unwrap().to_string(), v.clone())).collect::<Vec<(String, RV)>>();
                    IterationEnvironment::new(vec![GLOBAL_INTERNER.intern("0")], vec![RV::Object(RVObject::from_map(FxHashMap::from_iter(pairs.into_iter())))])
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
                    vec![GLOBAL_INTERNER.intern(&alias.to_string())],
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
