use rustc_hash::FxHashMap;

use crate::{
    engine::interpreter::{Aggregation, HaltReason},
    global::GLOBAL_INTERNER,
    plan::IntermediateExpr,
    value::{RV, iterator::ExecutionRow},
};

pub(crate) struct Grouper {
    group_exprs: Vec<IntermediateExpr>,
    aggregations: Vec<Aggregation>,
    interpreter: crate::engine::interpreter::Interpreter,
    groups: FxHashMap<Vec<RV>, Vec<Box<dyn Aggregator>>>,
}

impl Grouper {
    pub fn new(
        group_exprs: Vec<IntermediateExpr>,
        aggregators: Vec<Aggregation>,
        interpreter: crate::engine::interpreter::Interpreter,
    ) -> Grouper {
        Grouper {
            group_exprs,
            aggregations: aggregators,
            interpreter,
            groups: FxHashMap::default(),
        }
    }

    pub fn row(self: &mut Self, row: ExecutionRow) -> Result<(), HaltReason> {
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

            bucket_value[idx].as_mut().row(val);
        }

        Ok(())
    }

    pub fn finalize(&self) -> Vec<ExecutionRow> {
        let mut rows = vec![];

        for (_, agg) in self.groups.iter() {
            let mut row = ExecutionRow::new();
            for (idx, value) in agg.iter().enumerate() {
                let key = self.aggregations[idx].args[0].to_string();
                row.insert(GLOBAL_INTERNER.intern(&key), value.finalize());
            }

            rows.push(row);
        }

        rows
    }
}

pub trait Aggregator {
    fn row(self: &mut Self, row: RV);
    fn finalize(self: &Self) -> RV;
}
