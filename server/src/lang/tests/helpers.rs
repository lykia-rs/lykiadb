#[macro_export]
macro_rules! lexm {
    ($a: literal) => {
        Some($a.to_owned())
    };
}
