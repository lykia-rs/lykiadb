use lykiadb_lang::types::Datatype;
use rustc_hash::FxHashMap;

use crate::{
    engine::interpreter::Output, libs::stdlib::{arr::nt_create_arr, json::json, time::time, math::math, dtype::dtype, bench::bench}, lykia_lib, util::Shared, value::{
        RV,
        callable::{Function, RVCallable},
        object::RVObject,
    }
};

use self::{
    out::nt_print,
};

pub mod arr;
pub mod dtype;
pub mod json;
pub mod out;
pub mod time;
pub mod math;
pub mod bench;

lykia_lib!(std_core, vec![json(), time(), math(), dtype(), bench()]);

pub fn stdlib(out: Option<Shared<Output>>) -> FxHashMap<String, RV> {
    let mut std = std_core().as_raw();
    let mut io_namespace = FxHashMap::default();
    let mut arr_namespace = FxHashMap::default();

    io_namespace.insert(
        "print".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Native { function: nt_print },
            Datatype::Unknown,
            Datatype::Unit,
            
        )),
    );

    arr_namespace.insert(
        "new".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Native {
                function: nt_create_arr,
            },
            Datatype::Tuple(vec![Datatype::Num]),
            Datatype::Array(Box::new(Datatype::Num)),
            
        )),
    );

    if out.is_some() {
        let mut test_namespace = FxHashMap::default();

        test_namespace.insert(
            "out".to_owned(),
            RV::Callable(RVCallable::new(
                Function::Stateful(out.unwrap().clone()),
                Datatype::Unit,
                Datatype::Unit,
                
            )),
        );

        std.insert(
            "test_utils".to_owned(),
            RV::Object(RVObject::from_map(test_namespace)),
        );
    }

    std.insert(
        "io".to_owned(),
        RV::Object(RVObject::from_map(io_namespace)),
    );

    std.insert(
        "arr".to_owned(),
        RV::Object(RVObject::from_map(arr_namespace)),
    );

    std
}
