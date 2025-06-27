use crate::engine::{error::ExecutionError, interpreter::HaltReason};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use string_interner::symbol::SymbolU32;

use super::RV;
#[derive(Debug)]
pub struct EnvironmentFrame {
    map: RwLock<FxHashMap<SymbolU32, RV>>,
    pub parent: Option<Arc<EnvironmentFrame>>,
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

macro_rules! to_ancestor {
    ($init:expr, $distance:expr) => {{
        let mut env = $init;
        for _ in 0..$distance {
            env = env.parent.as_ref().unwrap();
        }
        env
    }};
}

impl From<EnvironmentError> for ExecutionError {
    fn from(err: EnvironmentError) -> Self {
        ExecutionError::Environment(err)
    }
}

impl EnvironmentFrame {
    pub fn new(parent: Option<Arc<EnvironmentFrame>>) -> EnvironmentFrame {
        EnvironmentFrame {
            parent,
            map: RwLock::new(FxHashMap::default()),
        }
    }

    pub fn define(&self, name: SymbolU32, value: RV) {
        self.map.write().unwrap().insert(name, value);
    }

    pub fn assign(&self, key: &str, key_sym: SymbolU32, value: RV) -> Result<bool, HaltReason> {
        if self.map.read().unwrap().contains_key(&key_sym) {
            self.map.write().unwrap().insert(key_sym, value);
            return Ok(true);
        }

        self.parent.as_ref().map_or(
            Err(HaltReason::Error(
                EnvironmentError::Other {
                    message: format!("Assignment to an undefined variable '{key}'"),
                }
                .into(),
            )),
            |parent| parent.as_ref().assign(key, key_sym, value),
        )
    }

    pub fn assign_at(
        env: &Arc<EnvironmentFrame>,
        distance: usize,
        key: &str,
        key_sym: SymbolU32,
        value: RV,
    ) -> Result<bool, HaltReason> {
        if distance == 0 {
            return env.assign(key, key_sym, value);
        }
        if distance == 1 && env.parent.is_some() {
            return env.parent.as_ref().unwrap().assign(key, key_sym, value);
        }
        to_ancestor!(env, distance).assign(key, key_sym, value)
    }

    pub fn read(&self, key: &str, key_sym: &SymbolU32) -> Result<RV, HaltReason> {
        let guard = self.map.read().unwrap();
        if let Some(value) = guard.get(key_sym) {
            // TODO(vck): Remove clone
            return Ok(value.clone());
        }
        self.parent.as_ref().map_or(
            Err(HaltReason::Error(
                EnvironmentError::Other {
                    message: format!("Variable '{key}' was not found"),
                }
                .into(),
            )),
            |parent| parent.read(key, key_sym),
        )
    }

    pub fn read_at(
        env: &Arc<EnvironmentFrame>,
        distance: usize,
        key: &str,
        key_sym: &SymbolU32,
    ) -> Result<RV, HaltReason> {
        if distance == 0 {
            return env.read(key, key_sym);
        }
        if distance == 1 && env.parent.is_some() {
            return env.parent.as_ref().unwrap().read(key, key_sym);
        }
        to_ancestor!(env, distance)
            .map
            .read()
            .unwrap()
            .get(key_sym)
            .map_or(
                Err(HaltReason::Error(
                    EnvironmentError::Other {
                        message: format!("Variable '{key}' was not found"),
                    }
                    .into(),
                )),
                |v| Ok(v.clone()),
            )
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use string_interner::{StringInterner, backend::StringBackend, symbol::SymbolU32};

    use crate::value::RV;

    fn get_interner() -> StringInterner<StringBackend<SymbolU32>> {
        StringInterner::<StringBackend<SymbolU32>>::new()
    }

    #[test]
    fn test_read_basic() {
        let env_man = super::EnvironmentFrame::new(None);
        let mut interner = get_interner();
        env_man.define(interner.get_or_intern("five"), RV::Num(5.0));
        assert_eq!(
            env_man
                .read("five", &interner.get_or_intern("five"))
                .unwrap(),
            RV::Num(5.0)
        );
    }

    #[test]
    fn test_read_from_parent() {
        let root = super::EnvironmentFrame::new(None);
        let mut interner = get_interner();
        root.define(interner.get_or_intern("five"), RV::Num(5.0));
        let child = super::EnvironmentFrame::new(Some(Arc::new(root)));
        assert_eq!(
            child.read("five", &interner.get_or_intern("five")).unwrap(),
            RV::Num(5.0)
        );
    }

    #[test]
    fn test_write_to_parent() {
        let root = Arc::new(super::EnvironmentFrame::new(None));
        let mut interner = get_interner();

        root.define(interner.get_or_intern("five"), RV::Num(5.0));

        let child = super::EnvironmentFrame::new(Some(root.clone()));

        child
            .assign("five", interner.get_or_intern("five"), RV::Num(5.1))
            .unwrap();

        assert_eq!(
            root.read("five", &interner.get_or_intern("five")).unwrap(),
            RV::Num(5.1)
        );

        assert_eq!(
            child.read("five", &interner.get_or_intern("five")).unwrap(),
            RV::Num(5.1)
        );
    }

    #[test]
    fn test_read_undefined_variable() {
        let env = super::EnvironmentFrame::new(None);
        let mut interner = get_interner();
        assert!(env.read("five", &interner.get_or_intern("five")).is_err());
    }

    #[test]
    fn test_assign_to_undefined_variable() {
        let env = super::EnvironmentFrame::new(None);
        let mut interner = get_interner();
        assert!(
            env.assign("five", interner.get_or_intern("five"), RV::Num(5.0))
                .is_err()
        );
    }
}
