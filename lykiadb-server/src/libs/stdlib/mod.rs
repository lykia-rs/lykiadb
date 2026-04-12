use lykiadb_common::memory::Shared;
use lykiadb_lang::types::Datatype;
use rustc_hash::FxHashMap;

use crate::{
    interpreter::output::Output,
    libs::stdlib::{
        arr::arr, bench::bench, dtype::dtype, json::json, math::math, out::out, time::time,
    },
    lykia_lib,
    value::{
        RV,
        callable::{Function, RVCallable},
        object::RVObject,
    },
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
    let std: std::collections::HashMap<String, RV<'_>, std::hash::BuildHasherDefault<rustc_hash::FxHasher>> = std_core().as_raw();
    std
}
