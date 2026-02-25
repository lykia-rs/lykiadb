pub mod comm;
pub mod interpreter;
pub mod exec;
pub mod global;
pub mod libs;
pub mod query;
pub mod util;
pub mod value;

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
