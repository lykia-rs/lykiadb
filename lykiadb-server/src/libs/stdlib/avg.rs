use crate::{exec::aggregation::Aggregator, lykia_agg, lykia_module, value::RV};

#[derive(Default)]
pub(crate) struct AvgAggregator {
    accumulator: f64,
    items: usize
}

impl Aggregator for AvgAggregator {
    fn row(self: &mut Self, expr_val: RV) {
        if let Some(n) = expr_val.as_number() {
            self.accumulator += n;
        }
        self.items += 1;
    }

    fn finalize(self: &Self) -> crate::value::RV {
        if self.items == 0 {
            return RV::Num(0.0);
        }

        RV::Num(self.accumulator / (self.items as f64))
    }
}

lykia_module!(math, {
    agg => lykia_agg!(AvgAggregator)
});