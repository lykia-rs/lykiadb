pub mod comm;
pub mod engine;
pub mod exec;
pub mod global;
pub mod plan;
pub mod util;
pub mod value;
pub mod libs;

#[macro_export]
macro_rules! assert_plan {
    ($($name:ident: {$field:literal => $value:literal}),*) => {
        $(
            #[test]
            fn $name() {
                expect_plan($field, $value);
            }
        )*
    };
}

#[macro_export]
macro_rules! lykia_lambda {
    ($builder:expr) => {
        crate::value::callable::Function::Lambda {
            function: $builder,
        }
    };
}

#[macro_export]
macro_rules! lykia_agg {
    ($agg:ident) => {
        crate::value::callable::Function::Agg {
            function: || Box::new($agg::default()),
        }
    };
}

#[macro_export]
macro_rules! lykia_module {
    ($name: ident, {$($function_name:ident=>$callable:expr),*}) => {
        use lykiadb_lang::types::Datatype;
        use rustc_hash::FxHashMap;
        use crate::{
            value::{
                callable::{CallableKind, RVCallable},
            }
        };
        pub fn $name() -> (String, RV) {
            let mut map = FxHashMap::default();
            $(
                map.insert(
                    stringify!($function_name).to_owned(),
                    RV::Callable(RVCallable::new(
                        $callable,
                        Datatype::Unknown,
                        Datatype::Unknown,
                        CallableKind::Generic,
                    )),
                );
            )*

            (stringify!($name).to_owned(), RV::Object(crate::value::object::RVObject::from_map(map)))
        }
    }
}

#[macro_export]
macro_rules! lykia_lib {
    ($name: ident, $value: expr) => {
        pub fn $name() -> FxHashMap<String, RV> {
            let mut lib = FxHashMap::default();
            for (key, val) in $value {
                lib.insert(key, val);
            }
            lib
        }
    }
}
