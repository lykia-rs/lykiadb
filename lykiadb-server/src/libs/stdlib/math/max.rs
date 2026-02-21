use crate::{exec::aggregation::Aggregator, value::RV};

#[derive(Default)]
pub(crate) struct MaxAggregator {
    value: Option<f64>,
}

impl<'exec> Aggregator<'exec> for MaxAggregator {
    fn row(&mut self, expr_val: &RV) {
        if let Some(n) = expr_val.as_number() {
            if self.value.is_none() {
                self.value = Some(n);
            } else if let Some(v) = self.value
                && n > v
            {
                self.value = Some(n);
            }
        }
    }

    fn finalize(&self) -> RV<'exec> {
        if let Some(n) = self.value {
            RV::Double(n)
        } else {
            RV::Undefined
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_aggregator() {
        let mut agg = MaxAggregator::default();

        agg.row(&RV::Double(10.0));
        agg.row(&RV::Double(30.0));
        agg.row(&RV::Double(20.0));

        assert_eq!(agg.finalize(), RV::Double(30.0));
    }

    #[test]
    fn test_max_aggregator_single_value() {
        let mut agg = MaxAggregator::default();
        agg.row(&RV::Double(42.0));
        assert_eq!(agg.finalize(), RV::Double(42.0));
    }

    #[test]
    fn test_max_aggregator_empty() {
        let agg = MaxAggregator::default();
        assert_eq!(agg.finalize(), RV::Undefined);
    }
}
