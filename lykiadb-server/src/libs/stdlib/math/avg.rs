use crate::{exec::aggregation::Aggregator, value::RV};

pub(crate) struct AvgAggregator {
    accumulator: f64,
    items: usize,
}

impl Default for AvgAggregator {
    fn default() -> Self {
        AvgAggregator {
            accumulator: 0.,
            items: 0,
        }
    }
}

impl<'rv> Aggregator<'rv> for AvgAggregator {
    fn row(&mut self, expr_val: &RV) {
        if let Some(n) = expr_val.as_double() {
            self.accumulator += n;
        }
        self.items += 1;
    }

    fn finalize(&self) -> crate::value::RV<'rv> {
        if self.items == 0 {
            return RV::Double(0.0);
        }

        RV::Double(self.accumulator / (self.items as f64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avg_aggregator() {
        let mut agg = AvgAggregator::default();

        agg.row(&RV::Double(10.0));
        agg.row(&RV::Double(20.0));
        agg.row(&RV::Double(30.0));

        assert_eq!(agg.finalize(), RV::Double(20.0));
    }

    #[test]
    fn test_avg_aggregator_empty() {
        let agg = AvgAggregator::default();
        assert_eq!(agg.finalize(), RV::Double(0.0));
    }

    #[test]
    fn test_avg_aggregator_with_non_numbers() {
        let mut agg = AvgAggregator::default();

        agg.row(&RV::Double(10.0));
        agg.row(&RV::Str(std::sync::Arc::new("not a number".to_string())));
        agg.row(&RV::Double(20.0));
        agg.row(&RV::Bool(true));

        assert_eq!(agg.finalize(), RV::Double(7.75)); // (10 + 20 + 1) / 4 items
    }
}
