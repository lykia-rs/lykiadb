#[macro_export]
macro_rules! lexm {
    ($a: literal) => {
        Rc::new($a.to_owned())
    };
}
