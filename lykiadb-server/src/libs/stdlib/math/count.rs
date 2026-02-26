use crate::value::{RV, callable::Aggregator};

#[derive(Default)]
pub(crate) struct CountAggregator {
    count: usize,
}

impl<'rv> Aggregator<'rv> for CountAggregator {
    fn row(&mut self, _expr_val: &RV) {
        self.count += 1;
    }

    fn finalize(&self) -> crate::value::RV<'rv> {
        RV::Double(self.count as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_aggregator() {
        let mut agg = CountAggregator::default();

        agg.row(&RV::Double(10.0));
        agg.row(&RV::Str(std::sync::Arc::new("hello".to_string())));
        agg.row(&RV::Bool(true));

        assert_eq!(agg.finalize(), RV::Double(3.0));
    }

    #[test]
    fn test_count_aggregator_empty() {
        let agg = CountAggregator::default();
        assert_eq!(agg.finalize(), RV::Double(0.0));
    }
}
