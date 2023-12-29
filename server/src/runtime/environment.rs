use crate::runtime::interpreter::HaltReason;
use crate::runtime::types::RV;
use crate::util::{alloc_shared, Shared};
use core::panic;
use rustc_hash::FxHashMap;

use super::interpreter::InterpretError;

#[derive(Debug)]
pub struct Environment {
    map: FxHashMap<String, RV>,
    pub parent: Option<Shared<Environment>>,
}

impl Environment {
    pub fn new(parent: Option<Shared<Environment>>) -> Shared<Environment> {
        alloc_shared(Environment {
            map: FxHashMap::default(),
            parent,
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
            return self
                .parent
                .as_mut()
                .unwrap()
                .borrow_mut()
                .assign(name, value);
        }
        Err(HaltReason::Error(InterpretError::Other {
            message: format!("Assignment to an undefined variable '{}'", &name),
        }))
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &str,
        value: RV,
    ) -> Result<bool, HaltReason> {
        let ancestor = self.ancestor(distance);

        if let Some(unwrapped) = ancestor {
            unwrapped.borrow_mut().map.insert(name.to_string(), value);
        } else {
            self.map.insert(name.to_string(), value);
        }

        Ok(true)
    }

    pub fn read(&self, name: &str) -> Result<RV, HaltReason> {
        if self.map.contains_key(name) {
            // TODO(vck): Remove clone
            return Ok(self.map.get(name).unwrap().clone());
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().borrow().read(name);
        }

        Err(HaltReason::Error(InterpretError::Other {
            message: format!("Variable '{}' was not found", &name),
        }))
    }

    pub fn read_at(&self, distance: usize, name: &str) -> Result<RV, HaltReason> {
        let ancestor = self.ancestor(distance);

        if let Some(unwrapped) = ancestor {
            // TODO(vck): Remove clone
            return Ok(unwrapped.borrow().map.get(name).unwrap().clone());
        }
        return Ok(self.map.get(name).unwrap().clone());
    }

    pub fn ancestor(&self, distance: usize) -> Option<Shared<Environment>> {
        if distance == 0 {
            return None;
        }
        if distance == 1 {
            return Some(self.parent.as_ref().unwrap().clone());
        }
        if self.parent.is_some() {
            let pref = self.parent.as_ref().unwrap().borrow_mut();
            return pref.ancestor(distance - 1);
        }
        panic!("Invalid variable distance.");
    }
}

#[cfg(test)]
mod test {
    use crate::runtime::types::RV;

    #[test]
    fn test_read_basic() {
        let env = super::Environment::new(None);
        env.borrow_mut().declare("five".to_string(), RV::Num(5.0));
        assert_eq!(env.borrow().read("five").unwrap(), RV::Num(5.0));
    }

    #[test]
    fn test_read_from_parent() {
        let parent = super::Environment::new(None);
        parent
            .borrow_mut()
            .declare("five".to_string(), RV::Num(5.0));
        let child = super::Environment::new(Some(parent.clone()));
        assert_eq!(child.borrow().read("five").unwrap(), RV::Num(5.0));
    }

    #[test]
    fn test_write_to_parent() {
        let parent = super::Environment::new(None);
        parent
            .borrow_mut()
            .declare("five".to_string(), RV::Num(5.0));
        let child = super::Environment::new(Some(parent.clone()));
        child
            .borrow_mut()
            .assign("five".to_string(), RV::Num(5.1))
            .unwrap();
        assert_eq!(parent.borrow().read("five").unwrap(), RV::Num(5.1));
        assert_eq!(child.borrow().read("five").unwrap(), RV::Num(5.1));
    }

    #[test]
    fn test_read_undefined_variable() {
        let env = super::Environment::new(None);
        assert!(env.borrow().read("five").is_err());
    }

    #[test]
    fn test_assign_to_undefined_variable() {
        let env = super::Environment::new(None);
        assert!(env
            .borrow_mut()
            .assign("five".to_string(), RV::Num(5.0))
            .is_err());
    }
}