use std::sync::{Arc, RwLock};

pub type Shared<T> = Arc<RwLock<T>>;

pub fn alloc_shared<T>(obj: T) -> Shared<T> {
    Arc::new(RwLock::new(obj))
}