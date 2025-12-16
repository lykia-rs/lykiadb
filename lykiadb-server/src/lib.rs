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
