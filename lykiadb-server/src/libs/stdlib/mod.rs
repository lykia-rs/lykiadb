use lykiadb_common::memory::Shared;
use lykiadb_lang::types::Datatype;
use rustc_hash::FxHashMap;

use crate::{
    interpreter::Output,
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

pub fn stdlib<'rv>(out: Option<Shared<Output<'rv>>>) -> FxHashMap<String, RV<'rv>> {
    let mut std = std_core().as_raw();

    if let Some(out) = out {
        let mut test_namespace = FxHashMap::default();

        test_namespace.insert(
            "print".to_owned(),
            RV::Callable(RVCallable::new(
                Function::Stateful(out.clone()),
                Datatype::Unit,
                Datatype::Unit,
            )),
        );

        std.insert(
            "testutil".to_owned(),
            RV::Object(RVObject::from_map(test_namespace)),
        );
    }

    std
}
