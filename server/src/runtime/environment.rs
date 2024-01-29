use crate::runtime::interpreter::HaltReason;
use crate::runtime::types::RV;
use core::panic;
use rustc_hash::FxHashMap;
use std::borrow::{Borrow, BorrowMut};

use super::interpreter::InterpretError;

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct EnvId(pub usize);

#[derive(Debug)]
struct EnvironmentFrame {
    map: FxHashMap<String, RV>,
    pub parent: Option<EnvId>,
}

#[derive(Debug)]
pub struct Environment {
    envs: Vec<EnvironmentFrame>,
}

impl Environment {
    pub fn new() -> Self {
        let mut arena = Environment { envs: vec![] };
        arena.push(None);
        arena
    }

    pub fn push(&mut self, parent: Option<EnvId>) -> EnvId {
        self.envs.push(EnvironmentFrame {
            map: FxHashMap::default(),
            parent,
        });
        EnvId(self.envs.len() - 1)
    }

    pub fn pop(&self, env_id: EnvId) -> EnvId {
        // TODO(vck): Remove the env for real
        self.envs[env_id.0].parent.unwrap()
    }

    pub fn top(&self) -> EnvId {
        EnvId(self.envs.len() - 1)
    }

    pub fn declare(&mut self, env_id: EnvId, name: String, value: RV) {
        self.envs[env_id.0].map.insert(name, value);
    }

    pub fn assign(&mut self, env_id: EnvId, name: String, value: RV) -> Result<bool, HaltReason> {
        let env = self.envs[env_id.0].borrow();
        if env.map.contains_key(&name) {
            self.envs[env_id.0].borrow_mut().map.insert(name, value);
            return Ok(true);
        }

        if env.parent.is_some() {
            return self.assign(env.parent.unwrap(), name, value);
        }
        Err(HaltReason::Error(InterpretError::Other {
            message: format!("Assignment to an undefined variable '{}'", &name),
        }))
    }

    pub fn assign_at(
        &mut self,
        env_id: EnvId,
        distance: usize,
        name: &str,
        value: RV,
    ) -> Result<bool, HaltReason> {
        let ancestor = self.ancestor(env_id, distance);

        if let Some(unwrapped) = ancestor {
            self.envs[unwrapped.0]
                .borrow_mut()
                .map
                .insert(name.to_string(), value);
        } else {
            self.envs[env_id.0]
                .borrow_mut()
                .map
                .insert(name.to_string(), value);
        }

        Ok(true)
    }

    pub fn read(&self, env_id: EnvId, name: &str) -> Result<RV, HaltReason> {
        if self.envs[env_id.0].map.contains_key(name) {
            // TODO(vck): Remove clone
            return Ok(self.envs[env_id.0].map.get(name).unwrap().clone());
        }

        if self.envs[env_id.0].parent.is_some() {
            return self.read(self.envs[env_id.0].parent.unwrap(), name);
        }

        Err(HaltReason::Error(InterpretError::Other {
            message: format!("Variable '{}' was not found", &name),
        }))
    }

    pub fn read_at(&self, env_id: EnvId, distance: usize, name: &str) -> Result<RV, HaltReason> {
        let ancestor = self.ancestor(env_id, distance);

        if let Some(unwrapped) = ancestor {
            // TODO(vck): Remove clone
            return Ok(self.envs[unwrapped.0].map.get(name).unwrap().clone());
        }
        return Ok(self.envs[env_id.0].map.get(name).unwrap().clone());
    }

    pub fn ancestor(&self, env_id: EnvId, distance: usize) -> Option<EnvId> {
        if distance == 0 {
            return None;
        }
        if distance == 1 {
            return Some(self.envs[env_id.0].parent.unwrap());
        }
        if self.envs[env_id.0].parent.is_some() {
            let pref = self.envs[env_id.0].parent.unwrap();
            return self.ancestor(pref, distance - 1);
        }
        panic!("Invalid variable distance.");
    }
}

#[cfg(test)]
mod test {
    use crate::runtime::types::RV;

    #[test]
    fn test_read_basic() {
        let mut env_arena = super::Environment::new();
        let env = env_arena.top();
        env_arena.declare(env, "five".to_string(), RV::Num(5.0));
        assert_eq!(env_arena.read(env, "five").unwrap(), RV::Num(5.0));
    }

    #[test]
    fn test_read_from_parent() {
        let mut env_arena = super::Environment::new();
        let parent = env_arena.top();
        env_arena.declare(parent, "five".to_string(), RV::Num(5.0));
        let child = env_arena.push(Some(parent));
        assert_eq!(env_arena.read(child, "five").unwrap(), RV::Num(5.0));
    }

    #[test]
    fn test_write_to_parent() {
        let mut env_arena = super::Environment::new();
        let parent = env_arena.top();
        env_arena.declare(parent, "five".to_string(), RV::Num(5.0));
        let child = env_arena.push(Some(parent));
        env_arena
            .assign(child, "five".to_string(), RV::Num(5.1))
            .unwrap();
        assert_eq!(env_arena.read(parent, "five").unwrap(), RV::Num(5.1));
        assert_eq!(env_arena.read(child, "five").unwrap(), RV::Num(5.1));
    }

    #[test]
    fn test_read_undefined_variable() {
        let env_arena = super::Environment::new();
        let env = env_arena.top();
        assert!(env_arena.read(env, "five").is_err());
    }

    #[test]
    fn test_assign_to_undefined_variable() {
        let mut env_arena = super::Environment::new();
        let env = env_arena.top();
        assert!(env_arena
            .assign(env, "five".to_string(), RV::Num(5.0))
            .is_err());
    }
}
