#[macro_export] macro_rules! lexm {
    ($a: literal) => {
        Some(Rc::new($a.to_owned()))
    };
}
