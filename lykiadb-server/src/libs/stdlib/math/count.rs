use crate::{exec::aggregation::Aggregator, value::RV};

pub(crate) struct CountAggregator {
    count: usize,
}

impl Default for CountAggregator {
    fn default() -> Self {
        CountAggregator { count: 0 }
    }
}

impl Aggregator for CountAggregator {
    fn row(&mut self, _expr_val: &RV) {
        self.count += 1;
    }

    fn finalize(&self) -> crate::value::RV {
        RV::Num(self.count as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_aggregator() {
        let mut agg = CountAggregator::default();

        agg.row(&RV::Num(10.0));
        agg.row(&RV::Str(std::sync::Arc::new("hello".to_string())));
        agg.row(&RV::Bool(true));

        assert_eq!(agg.finalize(), RV::Num(3.0));
    }

    #[test]
    fn test_count_aggregator_empty() {
        let agg = CountAggregator::default();
        assert_eq!(agg.finalize(), RV::Num(0.0));
    }
}
