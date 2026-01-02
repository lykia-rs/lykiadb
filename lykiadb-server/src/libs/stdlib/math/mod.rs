use crate::{
    libs::stdlib::math::{
        avg::AvgAggregator, count::CountAggregator, max::MaxAggregator, min::MinAggregator,
        sum::SumAggregator,
    },
    lykia_agg_fn, lykia_module,
};

mod avg;
mod count;
mod max;
mod min;
mod sum;

lykia_module!(math, {
    avg => lykia_agg_fn!(avg, AvgAggregator),
    sum => lykia_agg_fn!(sum, SumAggregator),
    count => lykia_agg_fn!(count, CountAggregator),
    min => lykia_agg_fn!(min, MinAggregator),
    max => lykia_agg_fn!(max, MaxAggregator)
}, {}, [avg, sum, count, min, max]);
