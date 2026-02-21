use rustc_hash::FxHashMap;

use crate::{
    engine::interpreter::{HaltReason, Interpreter},
    global::GLOBAL_INTERNER,
    plan::{Aggregation, IntermediateExpr},
    value::{RV, iterator::ExecutionRow},
};

pub(crate) struct Grouper<'session> {
    group_exprs: Vec<IntermediateExpr<'session>>,
    aggregations: Vec<Aggregation<'session>>,
    interpreter: Interpreter<'session>,
    groups: FxHashMap<Vec<RV<'session>>, Vec<Box<dyn Aggregator<'session>>>>,
}

impl<'session> Grouper<'session> {
    pub fn new(
        group_exprs: Vec<IntermediateExpr<'session>>,
        aggregators: Vec<Aggregation<'session>>,
        interpreter: Interpreter<'session>,
    ) -> Grouper<'session> {
        Grouper {
            group_exprs,
            aggregations: aggregators,
            interpreter,
            groups: FxHashMap::default(),
        }
    }

    pub fn row(&mut self, row: ExecutionRow<'session>) -> Result<(), HaltReason<'session>> {
        let mut bucket: Vec<RV> = vec![];

        for group_expr in self.group_exprs.iter() {
            bucket.push(match group_expr {
                IntermediateExpr::Constant(val) => val.clone(),
                IntermediateExpr::Expr { expr } => self.interpreter.eval_with_row(expr, &row)?,
            });
        }

        if !self.groups.contains_key(&bucket) {
            let mut aggregators: Vec<Box<dyn Aggregator>> = vec![];

            for aggregation in self.aggregations.iter() {
                aggregators.push(aggregation.callable.unwrap()())
            }

            self.groups.insert(bucket.clone(), aggregators);
        }

        let bucket_value = self.groups.get_mut(&bucket).unwrap();

        for (idx, agg) in self.aggregations.iter().enumerate() {
            let val = self.interpreter.eval_with_row(&agg.args[0], &row)?;

            bucket_value[idx].as_mut().row(&val);
        }

        Ok(())
    }

    pub fn finalize(&self) -> Vec<ExecutionRow<'session>> {
        let mut rows = vec![];

        for (bucket, agg) in self.groups.iter() {
            let mut row = ExecutionRow::new();
            for (idx, value) in bucket.iter().enumerate() {
                row.insert(GLOBAL_INTERNER.intern(&format!("col_{idx}")), value.clone());
            }
            for (idx, value) in agg.iter().enumerate() {
                row.insert(
                    GLOBAL_INTERNER.intern(&self.aggregations[idx].call_sign),
                    value.finalize(),
                );
            }

            rows.push(row);
        }

        rows
    }
}

pub trait Aggregator<'exec> {
    fn row(&mut self, row: &RV<'exec>);
    fn finalize(&self) -> RV<'exec>;
}
