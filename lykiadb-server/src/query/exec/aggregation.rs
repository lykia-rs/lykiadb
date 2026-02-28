use rustc_hash::FxHashMap;

use crate::{
    global::GLOBAL_INTERNER,
    interpreter::{HaltReason, expr::{ExprEngine, StatefulExprEngine}},
    query::plan::{Aggregation, IntermediateExpr},
    value::{RV, callable::Aggregator, iterator::ExecutionRow},
};

pub(crate) struct Grouper<'v> {
    group_exprs: Vec<IntermediateExpr<'v>>,
    aggregations: Vec<Aggregation<'v>>,
    expr_engine: &'v StatefulExprEngine<'v>,
    groups: FxHashMap<Vec<RV<'v>>, Vec<Box<dyn Aggregator<'v>>>>,
}

impl<'v> Grouper<'v> {
    pub fn new(
        group_exprs: Vec<IntermediateExpr<'v>>,
        aggregators: Vec<Aggregation<'v>>,
        expr_engine: &'v StatefulExprEngine<'v>,
    ) -> Grouper<'v> {
        Grouper {
            group_exprs,
            aggregations: aggregators,
            expr_engine,
            groups: FxHashMap::default(),
        }
    }

    pub fn row(&mut self, row: ExecutionRow<'v>) -> Result<(), HaltReason<'v>> {
        let mut bucket: Vec<RV> = vec![];

        for group_expr in self.group_exprs.iter() {
            bucket.push(match group_expr {
                IntermediateExpr::Constant(val) => val.clone(),
                IntermediateExpr::Expr { expr } => {
                    self.expr_engine.eval_with_exec_row(expr, row.clone())?
                }
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
            let val = self.expr_engine.eval_with_exec_row(&agg.args[0], row.clone())?;

            bucket_value[idx].as_mut().row(&val);
        }

        Ok(())
    }

    pub fn finalize(&self) -> Vec<ExecutionRow<'v>> {
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
