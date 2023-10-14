use core::panic;
use std::cell::RefCell;
use std::rc::Rc;
use rustc_hash::FxHashMap;
use crate::runtime::interpreter::HaltReason;
use crate::runtime::types::RV;

pub type Shared<T> = Rc<RefCell<T>>;

pub fn alloc_shared<T>(obj: T) -> Shared<T> {
    Rc::new(RefCell::new(obj))
}

#[derive(Debug)]
pub struct Environment {
    map: FxHashMap<String, RV>,
    pub parent: Option<Shared<Environment>>
}

impl Environment {
    pub fn new(parent: Option<Shared<Environment>>) -> Shared<Environment> {
        alloc_shared(Environment {
            map: FxHashMap::default(),
            parent
        })
    }

    pub fn pop(&mut self) -> Shared<Environment> {
        self.parent.clone().unwrap()
    }

    pub fn declare(&mut self, name: String, value: RV) {
        self.map.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: RV) -> Result<bool, HaltReason> {
        if self.map.contains_key(&name) {
            self.map.insert(name, value);
            return Ok(true);
        }

        if self.parent.is_some() {
            return self.parent.as_mut().unwrap().borrow_mut().assign(name, value);
        }

        Err(HaltReason::GenericError(format!("Assignment to an undefined variable '{}'", &name)))
    }

    pub fn read(&self, name: &String) -> Result<RV, HaltReason> {
        if self.map.contains_key(name) {
            // TODO(vck): Remove clone
            return Ok(self.map.get(name).unwrap().clone());
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().borrow().read(name);
        }

        Err(HaltReason::GenericError(format!("Variable '{}' was not found.", &name)))
    }

    pub fn read_at(&self, distance: usize, name: &str) -> Result<RV, HaltReason> {
        // TODO(vck): Remove clone
        let ancestor = self.ancestor(distance);
        if ancestor.is_some() {
            return Ok(ancestor.unwrap().borrow().map.get(name).unwrap().clone());
        }
        return Ok(self.map.get(name).unwrap().clone());
    }

    pub fn ancestor(&self, distance: usize) -> Option<Shared<Environment>> {
        if distance == 0 {
            return None;
        }
        if self.parent.is_some() {
            let pref = self.parent.as_ref().unwrap().borrow_mut();
            return pref.ancestor(distance - 1);
        }
        panic!("Ancestor not found");
    }
}