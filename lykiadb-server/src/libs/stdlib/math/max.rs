use crate::{exec::aggregation::Aggregator, value::RV};

#[derive(Default)]
pub(crate) struct MaxAggregator {
    value: Option<f64>,
}

impl Aggregator for MaxAggregator {
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

    fn finalize(&self) -> crate::value::RV {
        if let Some(n) = self.value {
            RV::Num(n)
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

        agg.row(&RV::Num(10.0));
        agg.row(&RV::Num(30.0));
        agg.row(&RV::Num(20.0));

        assert_eq!(agg.finalize(), RV::Num(30.0));
    }

    #[test]
    fn test_max_aggregator_single_value() {
        let mut agg = MaxAggregator::default();
        agg.row(&RV::Num(42.0));
        assert_eq!(agg.finalize(), RV::Num(42.0));
    }

    #[test]
    fn test_max_aggregator_empty() {
        let agg = MaxAggregator::default();
        assert_eq!(agg.finalize(), RV::Undefined);
    }
}
