use crate::engine::{error::ExecutionError, interpreter::HaltReason};
use core::panic;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, BorrowMut};
use string_interner::symbol::SymbolU32;

use super::RV;

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct EnvId(pub usize);

#[derive(Debug)]
struct EnvironmentFrame {
    map: FxHashMap<SymbolU32, RV>,
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

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
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
        let map: FxHashMap<SymbolU32, RV> = FxHashMap::default();
        // map.try_reserve(4).unwrap(); // speeds up sudo execution, don't know why
        self.envs.push(EnvironmentFrame { map, parent });
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

    pub fn declare(&mut self, env_id: EnvId, name: SymbolU32, value: RV) {
        self.envs[env_id.0].map.insert(name, value);
    }

    pub fn assign(
        &mut self,
        env_id: EnvId,
        key: &str,
        key_sym: SymbolU32,
        value: RV,
    ) -> Result<bool, HaltReason> {
        let env = self.envs[env_id.0].borrow();
        if env.map.contains_key(&key_sym) {
            self.envs[env_id.0].borrow_mut().map.insert(key_sym, value);
            return Ok(true);
        }

        if env.parent.is_some() {
            return self.assign(env.parent.unwrap(), key, key_sym, value);
        }
        Err(HaltReason::Error(
            EnvironmentError::Other {
                message: format!("Assignment to an undefined variable '{}'", key),
            }
            .into(),
        ))
    }

    pub fn assign_at(
        &mut self,
        env_id: EnvId,
        distance: usize,
        name: SymbolU32,
        value: RV,
    ) -> Result<bool, HaltReason> {
        let ancestor = self.ancestor(env_id, distance);

        if let Some(unwrapped) = ancestor {
            self.envs[unwrapped.0].borrow_mut().map.insert(name, value);
        } else {
            self.envs[env_id.0].borrow_mut().map.insert(name, value);
        }

        Ok(true)
    }

    pub fn read(&self, env_id: EnvId, key: &str, key_sym: &SymbolU32) -> Result<RV, HaltReason> {
        if self.envs[env_id.0].map.contains_key(key_sym) {
            // TODO(vck): Remove clone
            return Ok(self.envs[env_id.0].map.get(key_sym).unwrap().clone());
        }

        if self.envs[env_id.0].parent.is_some() {
            return self.read(self.envs[env_id.0].parent.unwrap(), key, key_sym);
        }

        Err(HaltReason::Error(
            EnvironmentError::Other {
                message: format!("Variable '{}' was not found", key),
            }
            .into(),
        ))
    }

    pub fn read_at(
        &self,
        env_id: EnvId,
        distance: usize,
        key: &str,
        key_sym: &SymbolU32,
    ) -> Result<RV, HaltReason> {
        let ancestor = self.ancestor(env_id, distance);

        if let Some(unwrapped) = ancestor {
            // TODO(vck): Remove clone
            return Ok(self.envs[unwrapped.0].map.get(key_sym).unwrap().clone());
        }
        if let Some(val) = self.envs[env_id.0].map.get(key_sym) {
            return Ok(val.clone());
        }

        Err(HaltReason::Error(
            EnvironmentError::Other {
                message: format!("Variable '{}' was not found", key),
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
    use string_interner::{backend::StringBackend, symbol::SymbolU32, StringInterner};

    use crate::value::RV;

    fn get_interner() -> StringInterner<StringBackend<SymbolU32>> {
        StringInterner::<StringBackend<SymbolU32>>::new()
    }

    #[test]
    fn test_read_basic() {
        let mut env_man = super::Environment::new();
        let mut interner = get_interner();
        let env = env_man.top();
        env_man.declare(env, interner.get_or_intern("five"), RV::Num(5.0));
        assert_eq!(
            env_man
                .read(env, "five", &interner.get_or_intern("five"))
                .unwrap(),
            RV::Num(5.0)
        );
    }

    #[test]
    fn test_read_from_parent() {
        let mut env_man = super::Environment::new();
        let mut interner = get_interner();
        let parent = env_man.top();
        env_man.declare(parent, interner.get_or_intern("five"), RV::Num(5.0));
        let child = env_man.push(Some(parent));
        assert_eq!(
            env_man
                .read(child, "five", &interner.get_or_intern("five"))
                .unwrap(),
            RV::Num(5.0)
        );
    }

    #[test]
    fn test_write_to_parent() {
        let mut env_man = super::Environment::new();
        let mut interner = get_interner();
        let parent = env_man.top();
        env_man.declare(parent, interner.get_or_intern("five"), RV::Num(5.0));
        let child = env_man.push(Some(parent));
        env_man
            .assign(child, "five", interner.get_or_intern("five"), RV::Num(5.1))
            .unwrap();
        assert_eq!(
            env_man
                .read(parent, "five", &interner.get_or_intern("five"))
                .unwrap(),
            RV::Num(5.1)
        );
        assert_eq!(
            env_man
                .read(child, "five", &interner.get_or_intern("five"))
                .unwrap(),
            RV::Num(5.1)
        );
    }

    #[test]
    fn test_read_undefined_variable() {
        let env_man = super::Environment::new();
        let mut interner = get_interner();
        let env = env_man.top();
        assert!(env_man
            .read(env, "five", &interner.get_or_intern("five"))
            .is_err());
    }

    #[test]
    fn test_assign_to_undefined_variable() {
        let mut env_man = super::Environment::new();
        let mut interner = get_interner();
        let env = env_man.top();
        assert!(env_man
            .assign(env, "five", interner.get_or_intern("five"), RV::Num(5.0))
            .is_err());
    }
}
