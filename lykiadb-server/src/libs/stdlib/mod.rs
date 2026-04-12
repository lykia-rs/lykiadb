use rustc_hash::FxHashMap;

use crate::{
    libs::stdlib::{
        arr::arr, bench::bench, dtype::dtype, json::json, math::math, out::out, time::time,
    },
    lykia_lib,
    value::RV,
};

mod arr;
mod bench;
mod dtype;
mod json;
mod math;
mod out;
mod time;

lykia_lib!(
    std_core,
    vec![json(), time(), math(), dtype(), bench(), out(), arr()]
);

pub fn stdlib<'rv>() -> FxHashMap<String, RV<'rv>> {
    std_core().as_raw()
}
