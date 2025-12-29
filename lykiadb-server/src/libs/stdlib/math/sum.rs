use crate::{exec::aggregation::Aggregator, value::RV};

pub(crate) struct SumAggregator {
    accumulator: f64,
}

impl Default for SumAggregator {
    fn default() -> Self {
        SumAggregator { accumulator: 0. }
    }
}

impl Aggregator for SumAggregator {
    fn row(&mut self, expr_val: &RV) {
        if let Some(n) = expr_val.as_number() {
            self.accumulator += n;
        }
    }

    fn finalize(&self) -> crate::value::RV {
        RV::Num(self.accumulator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_aggregator() {
        let mut agg = SumAggregator::default();

        agg.row(&RV::Num(10.0));
        agg.row(&RV::Num(20.0));
        agg.row(&RV::Num(30.0));

        assert_eq!(agg.finalize(), RV::Num(60.0));
    }

    #[test]
    fn test_sum_aggregator_empty() {
        let agg = SumAggregator::default();
        assert_eq!(agg.finalize(), RV::Num(0.0));
    }

    #[test]
    fn test_sum_aggregator_with_non_numbers() {
        let mut agg = SumAggregator::default();

        agg.row(&RV::Num(10.0));
        agg.row(&RV::Str(std::sync::Arc::new("not a number".to_string())));
        agg.row(&RV::Num(20.0));

        assert_eq!(agg.finalize(), RV::Num(30.0));
    }
}
