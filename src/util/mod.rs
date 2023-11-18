use std::{cell::RefCell, rc::Rc};

pub type Shared<T> = Rc<RefCell<T>>;

pub fn alloc_shared<T>(obj: T) -> Shared<T> {
    Rc::new(RefCell::new(obj))
}
