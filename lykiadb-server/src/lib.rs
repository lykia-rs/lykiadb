pub mod engine;
pub mod execution;
pub mod interpreter;
pub mod libs;
pub mod query;
pub mod store;
pub mod value;

#[macro_export]
macro_rules! register_tests {
    ($path:literal) => {
        #[cfg(test)]
        mod lyqltests {
            use $crate::execution::session::SessionTester;
            use test_each_file::test_each_path;
            test_each_path! {
                in $path => {
                    |path: &std::path::Path| {
                        let input = std::fs::read_to_string(path).expect("Failed to read test file");
                        lykiadb_common::testing::TestRunner::new(Box::new(|| Box::new(SessionTester::new())))
                            .test_file_named(path.to_str().unwrap_or(""), &input)
                    }
                }
            }
        }
    };
}

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
macro_rules! lykia_native_fn {
    ($builder:expr) => {
        $crate::value::callable::Function::Native { function: $builder }
    };
}

#[macro_export]
macro_rules! lykia_agg_fn {
    ($name:ident, $agg:ident) => {
        $crate::value::callable::Function::Agg {
            name: stringify!($name).into(),
            function: || Box::new($agg::default()),
        }
    };
}

#[macro_export]
macro_rules! lykia_module {
    ($name: ident, {$($function_name:ident=>$callable:expr),*}, {$($constant_name:ident=>$constant:expr),*}, [$($root_name:ident),*]) => {
        use lykiadb_lang::types::Datatype;
        use $crate::libs::LykiaModule;
        use $crate::value::callable::RVCallable;

        pub fn $name<'v>() -> LykiaModule<'v> {
            let mut modl = LykiaModule::new(stringify!($name));
            $(
                modl.insert(
                    stringify!($function_name),
                    RVCallable::new(
                        $callable,
                        Datatype::Unknown,
                        Datatype::Unknown,
                    ),
                );
            )*

            $(
                modl.insert_raw(
                    stringify!($constant_name),
                    $constant,
                );
            )*

            $(
                modl.expose_as_root(stringify!($root_name));
            )*

            modl
        }
    }
}

#[macro_export]
macro_rules! lykia_lib {
    ($name: ident, $value: expr) => {
        use $crate::libs::LykiaLibrary;

        pub fn $name<'v>() -> LykiaLibrary<'v> {
            LykiaLibrary::new(stringify!($name), $value)
        }
    };
}
