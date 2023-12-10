use std::rc::Rc;

use rustc_hash::FxHashMap;

use self::{
    fib::nt_fib,
    json::{nt_json_decode, nt_json_encode},
    out::nt_print,
    time::nt_clock,
};

use super::types::{Function, RV};

pub mod fib;
pub mod json;
pub mod out;
pub mod time;

pub fn stdlib() -> FxHashMap<String, RV> {
    let mut std = FxHashMap::default();

    let mut benchmark_namespace = FxHashMap::default();
    let mut json_namespace = FxHashMap::default();
    let mut time_namespace = FxHashMap::default();

    benchmark_namespace.insert(
        "fib".to_owned(),
        RV::Callable(Some(1), Rc::new(Function::Lambda { function: nt_fib })),
    );

    json_namespace.insert(
        "stringify".to_owned(),
        RV::Callable(
            Some(1),
            Rc::new(Function::Lambda {
                function: nt_json_encode,
            }),
        ),
    );

    json_namespace.insert(
        "parse".to_owned(),
        RV::Callable(
            Some(1),
            Rc::new(Function::Lambda {
                function: nt_json_decode,
            }),
        ),
    );

    time_namespace.insert(
        "clock".to_owned(),
        RV::Callable(Some(0), Rc::new(Function::Lambda { function: nt_clock })),
    );

    std.insert("Benchmark".to_owned(), RV::Object(benchmark_namespace));
    std.insert("JSON".to_owned(), RV::Object(json_namespace));
    std.insert("Time".to_owned(), RV::Object(time_namespace));
    std.insert(
        "print".to_owned(),
        RV::Callable(None, Rc::new(Function::Lambda { function: nt_print })),
    );

    std
}
