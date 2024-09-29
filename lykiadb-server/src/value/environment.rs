use crate::engine::{error::ExecutionError, interpreter::HaltReason};
use core::panic;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, BorrowMut};

use super::RV;

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

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentError {
    /*AssignmentToUndefined {
        token: Token,
    },
    VariableNotFound {
        token: Token,
    },*/
    Other { message: String },
}

impl From<EnvironmentError> for ExecutionError {
    fn from(err: EnvironmentError) -> Self {
        ExecutionError::Environment(err)
    }
}

impl Environment {
    pub fn new() -> Self {
        let mut instance = Environment { envs: vec![] };
        instance.push(None);
        instance
    }

    pub fn push(&mut self, parent: Option<EnvId>) -> EnvId {
        self.envs.push(EnvironmentFrame {
            map: FxHashMap::default(),
            parent,
        });
        EnvId(self.envs.len() - 1)
    }

    pub fn remove(&mut self, env_id: EnvId) -> EnvId {
        let parent = self.envs[env_id.0].parent.unwrap();
        self.envs.remove(env_id.0);
        parent
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
        Err(HaltReason::Error(
            EnvironmentError::Other {
                message: format!("Assignment to an undefined variable '{}'", &name),
            }
            .into(),
        ))
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

        Err(HaltReason::Error(
            EnvironmentError::Other {
                message: format!("Variable '{}' was not found", &name),
            }
            .into(),
        ))
    }

    pub fn read_at(&self, env_id: EnvId, distance: usize, name: &str) -> Result<RV, HaltReason> {
        let ancestor = self.ancestor(env_id, distance);

        if let Some(unwrapped) = ancestor {
            // TODO(vck): Remove clone
            return Ok(self.envs[unwrapped.0].map.get(name).unwrap().clone());
        }
        if let Some(val) = self.envs[env_id.0].map.get(name) {
            return Ok(val.clone());
        }

        Err(HaltReason::Error(
            EnvironmentError::Other {
                message: format!("Variable '{}' was not found", &name),
            }
            .into(),
        ))
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
    use crate::value::RV;

    #[test]
    fn test_read_basic() {
        let mut env_man = super::Environment::new();
        let env = env_man.top();
        env_man.declare(env, "five".to_string(), RV::Num(5.0));
        assert_eq!(env_man.read(env, "five").unwrap(), RV::Num(5.0));
    }

    #[test]
    fn test_read_from_parent() {
        let mut env_man = super::Environment::new();
        let parent = env_man.top();
        env_man.declare(parent, "five".to_string(), RV::Num(5.0));
        let child = env_man.push(Some(parent));
        assert_eq!(env_man.read(child, "five").unwrap(), RV::Num(5.0));
    }

    #[test]
    fn test_write_to_parent() {
        let mut env_man = super::Environment::new();
        let parent = env_man.top();
        env_man.declare(parent, "five".to_string(), RV::Num(5.0));
        let child = env_man.push(Some(parent));
        env_man
            .assign(child, "five".to_string(), RV::Num(5.1))
            .unwrap();
        assert_eq!(env_man.read(parent, "five").unwrap(), RV::Num(5.1));
        assert_eq!(env_man.read(child, "five").unwrap(), RV::Num(5.1));
    }

    #[test]
    fn test_read_undefined_variable() {
        let env_man = super::Environment::new();
        let env = env_man.top();
        assert!(env_man.read(env, "five").is_err());
    }

    #[test]
    fn test_assign_to_undefined_variable() {
        let mut env_man = super::Environment::new();
        let env = env_man.top();
        assert!(env_man
            .assign(env, "five".to_string(), RV::Num(5.0))
            .is_err());
    }
}
