use dtype::{nt_array_of, nt_callable_of, nt_object_of, nt_tuple_of};
use lykiadb_lang::types::Datatype;
use rustc_hash::FxHashMap;

use crate::{
    util::Shared,
    value::{
        RV, object::RVObject, callable::{RVCallable, CallableKind, Function}
    },
};

use self::{
    dtype::nt_of,
    fib::nt_fib,
    json::{nt_json_decode, nt_json_encode},
    out::nt_print,
    time::nt_clock,
};

use super::interpreter::Output;

pub mod dtype;
pub mod fib;
pub mod json;
pub mod out;
pub mod time;

pub fn stdlib(out: Option<Shared<Output>>) -> FxHashMap<String, RV> {
    let mut std = FxHashMap::default();

    let mut benchmark_namespace = FxHashMap::default();
    let mut json_namespace = FxHashMap::default();
    let mut time_namespace = FxHashMap::default();
    let mut io_namespace = FxHashMap::default();
    let mut dtype_namespace = FxHashMap::default();

    benchmark_namespace.insert(
        "fib".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda { function: nt_fib },
            Datatype::Tuple(vec![Datatype::Num]),
            Datatype::Num,
            CallableKind::Generic,
        )),
    );

    json_namespace.insert(
        "stringify".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda {
                function: nt_json_encode,
            },
            Datatype::Unknown,
            Datatype::Str,
            CallableKind::Generic,
        )),
    );

    json_namespace.insert(
        "parse".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda {
                function: nt_json_decode,
            },
            Datatype::Str,
            // TODO(vck): This should be a concrete type
            Datatype::Unknown,
            CallableKind::Generic,
        )),
    );

    time_namespace.insert(
        "clock".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda { function: nt_clock },
            Datatype::Unit,
            Datatype::Num,
            CallableKind::Generic,
        )),
    );

    io_namespace.insert(
        "print".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda { function: nt_print },
            Datatype::Unknown,
            Datatype::Unit,
            CallableKind::Generic,
        )),
    );

    dtype_namespace.insert(
        "of_".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda { function: nt_of },
            Datatype::Unknown,
            Datatype::Datatype,
            CallableKind::Generic,
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
            CallableKind::Generic,
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
            CallableKind::Generic,
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
            CallableKind::Generic,
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
            CallableKind::Generic,
        )),
    );

    dtype_namespace.insert("dtype".to_owned(), RV::Datatype(Datatype::Datatype));

    dtype_namespace.insert("none".to_owned(), RV::Datatype(Datatype::None));

    if out.is_some() {
        let mut test_namespace = FxHashMap::default();

        test_namespace.insert(
            "out".to_owned(),
            RV::Callable(RVCallable::new(
                Function::Stateful(out.unwrap().clone()),
                Datatype::Unit,
                Datatype::Unit,
                CallableKind::Generic,
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
    std.insert("json".to_owned(), RV::Object(RVObject::from_map(json_namespace)));
    std.insert("time".to_owned(), RV::Object(RVObject::from_map(time_namespace)));
    std.insert("io".to_owned(), RV::Object(RVObject::from_map(io_namespace)));
    std.insert(
        "dtype".to_owned(),
        RV::Object(RVObject::from_map(dtype_namespace)),
    );

    std.insert(
        "avg".to_owned(),
        RV::Callable(RVCallable::new(
            Function::Lambda {
                function: nt_tuple_of,
            },
            Datatype::Unknown,
            Datatype::Unknown,
            CallableKind::Aggregator("avg".to_owned()),
        )),
    );

    std
}
