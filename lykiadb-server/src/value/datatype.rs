use rustc_hash::FxHashMap;

pub enum Datatype {
    Str,
    Num,
    Bool,
    Composite(FxHashMap<String, Datatype>),
    Array(Box<Datatype>),
    Callable,
    Undefined
}
