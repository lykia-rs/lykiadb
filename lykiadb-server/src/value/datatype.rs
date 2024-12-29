use rustc_hash::FxHashMap;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Datatype {
    Str,
    Num,
    Bool,
    Document(FxHashMap<String, Datatype>),
    Array(Box<Datatype>),
    Callable,
    Datatype,
    None
}

impl Display for Datatype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Datatype::Str => write!(f, "dtype::str"),
            Datatype::Num => write!(f, "dtype::num"),
            Datatype::Bool => write!(f, "dtype::bool"),
            Datatype::Document(map) => {
                write!(f, "dtype::document({{")?;
                for (key, value) in map.iter() {
                    writeln!(f, "{}: {}, ", key, value)?;
                }
                write!(f, "}})")
            },
            Datatype::Array(inner) => write!(f, "dtype::array({})", inner),
            Datatype::Callable => write!(f, "dtype::callable"),
            Datatype::Datatype => write!(f, "dtype::dtype"),
            Datatype::None => write!(f, "dtype::none")
        }
    }
}