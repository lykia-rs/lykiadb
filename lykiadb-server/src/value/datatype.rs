use rustc_hash::FxHashMap;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Datatype {
    Str,
    Num,
    Bool,
    Object(FxHashMap<String, Datatype>),
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
            Datatype::Object(map) => {
                write!(f, "dtype::object({{")?;
                write!(f, "{}", 
                    map.iter().map(|(key, value)| {
                        return format!("{}: {}", key, value);
                    }).collect::<Vec<String>>().join(", ")
                )?;
                write!(f, "}})")
            },
            Datatype::Array(inner) => write!(f, "dtype::array({})", inner),
            Datatype::Callable => write!(f, "dtype::callable"),
            Datatype::Datatype => write!(f, "dtype::dtype"),
            Datatype::None => write!(f, "dtype::none")
        }
    }
}