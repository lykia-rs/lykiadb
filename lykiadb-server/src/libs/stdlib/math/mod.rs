use crate::{
    libs::stdlib::math::{
        avg::AvgAggregator, count::CountAggregator, max::MaxAggregator, min::MinAggregator,
        sum::SumAggregator,
    },
    lykia_agg_fn, lykia_module, lykia_native_fn,
};

mod avg;
mod count;
mod max;
mod min;
mod sum;
mod modulo;

lykia_module!(math, {
    avg => lykia_agg_fn!(avg, AvgAggregator),
    sum => lykia_agg_fn!(sum, SumAggregator),
    count => lykia_agg_fn!(count, CountAggregator),
    min => lykia_agg_fn!(min, MinAggregator),
    max => lykia_agg_fn!(max, MaxAggregator),
    mod => lykia_native_fn!(modulo::nt_modulo)
}, {}, [avg, sum, count, min, max, mod]);
