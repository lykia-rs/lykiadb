use rustc_hash::FxHashMap;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Datatype {
    Str,
    Num,
    Bool,
    Object(FxHashMap<String, Datatype>),
    Array(Box<Datatype>),
    Tuple(Vec<Datatype>),
    Callable(Box<Datatype>, Box<Datatype>),
    Datatype,
    Unit,
    Unknown,
    None,
}

impl Display for Datatype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Datatype::Str => write!(f, "dtype::str"),
            Datatype::Num => write!(f, "dtype::num"),
            Datatype::Bool => write!(f, "dtype::bool"),
            Datatype::Object(map) => {
                write!(f, "dtype::object({{")?;
                write!(
                    f,
                    "{}",
                    map.iter()
                        .map(|(key, value)| { format!("{}: {}", key, value) })
                        .collect::<Vec<String>>()
                        .join(", ")
                )?;
                write!(f, "}})")
            }
            Datatype::Array(inner) => write!(f, "dtype::array({})", inner),
            Datatype::Callable(input, output) => {
                write!(f, "dtype::callable({}, {})", input, output)
            }
            Datatype::Datatype => write!(f, "dtype::dtype"),
            Datatype::Tuple(inner) => {
                write!(f, "dtype::tuple(")?;
                write!(
                    f,
                    "{}",
                    inner
                        .iter()
                        .map(|dtype| format!("{}", dtype))
                        .collect::<Vec<String>>()
                        .join(", ")
                )?;
                write!(f, ")")
            }
            Datatype::None => write!(f, "dtype::none"),
            Datatype::Unit => write!(f, "dtype::unit"),
            Datatype::Unknown => write!(f, "dtype::unknown"),
        }
    }
}
