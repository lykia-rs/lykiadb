use std::{cell::RefCell, sync::Arc};

pub type Shared<T> = Arc<RefCell<T>>;

pub fn alloc_shared<T>(obj: T) -> Shared<T> {
    Arc::new(RefCell::new(obj))
}
