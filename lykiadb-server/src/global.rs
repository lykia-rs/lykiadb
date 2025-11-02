use interb::Interner;
use once_cell::sync::Lazy;

// 1. Create a global, thread-safe Interner instance.
pub(crate) static GLOBAL_INTERNER: Lazy<Interner<'static>> = Lazy::new(|| Interner::with_capacity(1024));
