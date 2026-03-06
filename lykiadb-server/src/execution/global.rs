use interb::{Interner, Symbol};
use once_cell::sync::Lazy;

// 1. Create a global, thread-safe Interner instance.
pub(crate) static GLOBAL_INTERNER: Lazy<Interner<'static>> =
    Lazy::new(|| Interner::with_capacity(1024));

pub(crate) fn intern_string(string: &str) -> Symbol {
    GLOBAL_INTERNER.intern(string)
}
