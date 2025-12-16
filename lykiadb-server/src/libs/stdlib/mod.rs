use dtype::{nt_array_of, nt_callable_of, nt_object_of, nt_tuple_of};
use lykiadb_lang::types::Datatype;
use rustc_hash::FxHashMap;

use crate::{
    engine::interpreter::Output, libs::stdlib::{arr::nt_create_arr, json::json, time::time}, lykia_lib, util::Shared, value::{
        RV,
        callable::{Function, RVCallable},
        object::RVObject,
    }
};

use self::{
    dtype::nt_of,
    fib::nt_fib,
    out::nt_print,
};


pub mod arr;
pub mod dtype;
pub mod fib;
pub mod json;
pub mod out;
pub mod time;
pub mod math;

lykia_lib!(std_core, [json(), time()]);

pub fn stdlib(out: Option<Shared<Output>>) -> FxHashMap<String, RV> {
    let mut std = std_core();

    let mut benchmark_namespace = FxHashMap::default();
    let mut io_namespace = FxHashMap::default();
    let mut dtype_namespace = FxHashMap::default();
    let mut arr_namespace = FxHashMap::default();

    benchmark_namespace.insert(
        "fib".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda { function: nt_fib },
            Datatype::Tuple(vec![Datatype::Num]),
            Datatype::Num,
            
        )),
    );

    io_namespace.insert(
        "print".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda { function: nt_print },
            Datatype::Unknown,
            Datatype::Unit,
            
        )),
    );

    dtype_namespace.insert(
        "of_".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda { function: nt_of },
            Datatype::Unknown,
            Datatype::Datatype,
            
        )),
    );

    dtype_namespace.insert("str".to_owned(), RV::Datatype(Datatype::Str));

    dtype_namespace.insert("num".to_owned(), RV::Datatype(Datatype::Num));

    dtype_namespace.insert("bool".to_owned(), RV::Datatype(Datatype::Bool));

    dtype_namespace.insert("unit".to_owned(), RV::Datatype(Datatype::Unit));

    dtype_namespace.insert(
        "array".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda {
                function: nt_array_of,
            },
            Datatype::Unknown,
            Datatype::Datatype,
        )),
    );

    dtype_namespace.insert(
        "object".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda {
                function: nt_object_of,
            },
            Datatype::Unknown,
            Datatype::Datatype,
        )),
    );

    dtype_namespace.insert(
        "callable".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda {
                function: nt_callable_of,
            },
            Datatype::Unknown,
            Datatype::Datatype,
        )),
    );

    dtype_namespace.insert(
        "tuple".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda {
                function: nt_tuple_of,
            },
            Datatype::Unknown,
            Datatype::Datatype,
        )),
    );

    dtype_namespace.insert("dtype".to_owned(), RV::Datatype(Datatype::Datatype));

    dtype_namespace.insert("none".to_owned(), RV::Datatype(Datatype::None));

    arr_namespace.insert(
        "new".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda {
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
        "Benchmark".to_owned(),
        RV::Object(RVObject::from_map(benchmark_namespace)),
    );
    std.insert(
        "io".to_owned(),
        RV::Object(RVObject::from_map(io_namespace)),
    );
    std.insert(
        "dtype".to_owned(),
        RV::Object(RVObject::from_map(dtype_namespace)),
    );

    std.insert(
        "avg".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Agg {
                name: "avg".to_owned(),
                function: || Box::new(math::AvgAggregator::default()),
            },
            Datatype::Unknown,
            Datatype::Unknown,
        )),
    );

    std.insert(
        "arr".to_owned(),
        RV::Object(RVObject::from_map(arr_namespace)),
    );

    std
}
