use std::sync::Arc;

use callable::Callable;
use rustc_hash::FxHashMap;

use crate::util::Shared;

pub mod environment;
pub mod types;
pub mod callable;
#[derive(Debug, Clone)]
pub enum RV {
    Str(Arc<String>),
    Num(f64),
    Bool(bool),
    Object(Shared<FxHashMap<String, RV>>),
    Array(Shared<Vec<RV>>),
    Callable(Callable),
    Undefined,
    NaN,
    Null,
}