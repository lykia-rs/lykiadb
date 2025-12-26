use crate::{exec::aggregation::Aggregator, lykia_agg_fn, lykia_module, value::RV};

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

impl Aggregator for AvgAggregator {
    fn row(&mut self, expr_val: &RV) {
        if let Some(n) = expr_val.as_number() {
            self.accumulator += n;
        }
        self.items += 1;
    }

    fn finalize(&self) -> crate::value::RV {
        if self.items == 0 {
            return RV::Num(0.0);
        }

        RV::Num(self.accumulator / (self.items as f64))
    }
}

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

pub(crate) struct MinAggregator {
    value: Option<f64>,
}

impl Default for MinAggregator {
    fn default() -> Self {
        MinAggregator { value: None }
    }
}

impl Aggregator for MinAggregator {
    fn row(&mut self, expr_val: &RV) {
        if let Some(n) = expr_val.as_number() {
            if self.value.is_none() {
                self.value = Some(n);
            }
            else if let Some(v) = self.value
                && n < v
            {
                self.value = Some(n);
            }
        }
    }

    fn finalize(&self) -> crate::value::RV {
        if let Some(n) = self.value {
            RV::Num(n)
        }
        else {
            RV::Undefined
        }
    }
}

pub(crate) struct MaxAggregator {
    value: Option<f64>,
}

impl Default for MaxAggregator {
    fn default() -> Self {
        MaxAggregator { value: None }
    }
}

impl Aggregator for MaxAggregator {
    fn row(&mut self, expr_val: &RV) {
        if let Some(n) = expr_val.as_number() {
            if self.value.is_none() {
                self.value = Some(n);
            }
            else if let Some(v) = self.value
                && n > v
            {
                self.value = Some(n);
            }
        }
    }

    fn finalize(&self) -> crate::value::RV {
        if let Some(n) = self.value {
            RV::Num(n)
        }
        else {
            RV::Undefined
        }
    }
}

lykia_module!(math, {
    avg => lykia_agg_fn!(avg, AvgAggregator),
    sum => lykia_agg_fn!(sum, SumAggregator),
    min => lykia_agg_fn!(min, MinAggregator),
    max => lykia_agg_fn!(max, MaxAggregator)
}, {}, [avg, sum, min, max]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avg_aggregator() {
        let mut agg = AvgAggregator::default();

        agg.row(&RV::Num(10.0));
        agg.row(&RV::Num(20.0));
        agg.row(&RV::Num(30.0));

        assert_eq!(agg.finalize(), RV::Num(20.0));
    }

    #[test]
    fn test_avg_aggregator_empty() {
        let agg = AvgAggregator::default();
        assert_eq!(agg.finalize(), RV::Num(0.0));
    }

    #[test]
    fn test_avg_aggregator_with_non_numbers() {
        let mut agg = AvgAggregator::default();

        agg.row(&RV::Num(10.0));
        agg.row(&RV::Str(std::sync::Arc::new("not a number".to_string())));
        agg.row(&RV::Num(20.0));
        agg.row(&RV::Bool(true));

        assert_eq!(agg.finalize(), RV::Num(7.75)); // (10 + 20 + 1) / 4 items
    }

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

    #[test]
    fn test_min_aggregator() {
        let mut agg = MinAggregator::default();

        agg.row(&RV::Num(30.0));
        agg.row(&RV::Num(10.0));
        agg.row(&RV::Num(20.0));

        assert_eq!(agg.finalize(), RV::Num(10.0));
    }

    #[test]
    fn test_min_aggregator_single_value() {
        let mut agg = MinAggregator::default();
        agg.row(&RV::Num(42.0));
        assert_eq!(agg.finalize(), RV::Num(42.0));
    }

    #[test]
    fn test_min_aggregator_empty() {
        let agg = MinAggregator::default();
        assert_eq!(agg.finalize(), RV::Undefined);
    }

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

    #[test]
    fn test_aggregators_with_negative_numbers() {
        let mut min_agg = MinAggregator::default();
        let mut max_agg = MaxAggregator::default();
        let mut sum_agg = SumAggregator::default();
        let mut avg_agg = AvgAggregator::default();

        let values = vec![RV::Num(-10.0), RV::Num(5.0), RV::Num(-3.0)];

        for val in values {
            min_agg.row(&val);
            max_agg.row(&val);
            sum_agg.row(&val);
            avg_agg.row(&val);
        }

        assert_eq!(min_agg.finalize(), RV::Num(-10.0));
        assert_eq!(max_agg.finalize(), RV::Num(5.0));
        assert_eq!(sum_agg.finalize(), RV::Num(-8.0));
        assert_eq!(avg_agg.finalize(), RV::Num(-8.0 / 3.0));
    }
}
