use crate::{query::exec::aggregation::Aggregator, value::RV};

pub(crate) struct SumAggregator {
    accumulator: f64,
}

impl Default for SumAggregator {
    fn default() -> Self {
        SumAggregator { accumulator: 0. }
    }
}

impl<'rv> Aggregator<'rv> for SumAggregator {
    fn row(&mut self, expr_val: &RV) {
        if let Some(n) = expr_val.as_double() {
            self.accumulator += n;
        }
    }

    fn finalize(&self) -> crate::value::RV<'rv> {
        RV::Double(self.accumulator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_aggregator() {
        let mut agg = SumAggregator::default();

        agg.row(&RV::Double(10.0));
        agg.row(&RV::Double(20.0));
        agg.row(&RV::Double(30.0));

        assert_eq!(agg.finalize(), RV::Double(60.0));
    }

    #[test]
    fn test_sum_aggregator_empty() {
        let agg = SumAggregator::default();
        assert_eq!(agg.finalize(), RV::Double(0.0));
    }

    #[test]
    fn test_sum_aggregator_with_non_numbers() {
        let mut agg = SumAggregator::default();

        agg.row(&RV::Double(10.0));
        agg.row(&RV::Str(std::sync::Arc::new("not a number".to_string())));
        agg.row(&RV::Double(20.0));

        assert_eq!(agg.finalize(), RV::Double(30.0));
    }
}
