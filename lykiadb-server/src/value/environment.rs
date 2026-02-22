use crate::engine::{error::ExecutionError, interpreter::HaltReason};
use interb::Symbol;
use lykiadb_common::error::InputError;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use super::RV;
#[derive(Debug)]
pub struct EnvironmentFrame<'v> {
    map: RwLock<FxHashMap<Symbol, RV<'v>>>,
    pub parent: Option<Arc<EnvironmentFrame<'v>>>,
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

impl<'v> EnvironmentFrame<'v> {
    pub fn new(parent: Option<Arc<EnvironmentFrame<'v>>>) -> EnvironmentFrame<'v> {
        EnvironmentFrame {
            parent,
            map: RwLock::new(FxHashMap::default()),
        }
    }

    pub fn define(&self, name: Symbol, value: RV<'v>) {
        self.map.write().unwrap().insert(name, value);
    }

    pub fn assign(
        &self,
        key: &str,
        key_sym: Symbol,
        value: RV<'v>,
    ) -> Result<bool, HaltReason<'v>> {
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
        env: &Arc<EnvironmentFrame<'v>>,
        distance: usize,
        key: &str,
        key_sym: Symbol,
        value: RV<'v>,
    ) -> Result<bool, HaltReason<'v>> {
        if distance == 0 {
            return env.assign(key, key_sym, value);
        }
        if distance == 1 && env.parent.is_some() {
            return env.parent.as_ref().unwrap().assign(key, key_sym, value);
        }
        to_ancestor!(env, distance).assign(key, key_sym, value)
    }

    pub fn read(&self, key: &str, key_sym: &Symbol) -> Result<RV<'v>, HaltReason<'v>> {
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
        env: &Arc<EnvironmentFrame<'v>>,
        distance: usize,
        key: &str,
        key_sym: &Symbol,
    ) -> Result<RV<'v>, HaltReason<'v>> {
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

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentError {
    /*AssignmentToUndefined {
        token: Token,
    },
    VariableNotFound {
        token: Token,
    },*/
    #[error("{message}")]
    Other { message: String },
}

impl From<EnvironmentError> for ExecutionError {
    fn from(err: EnvironmentError) -> Self {
        ExecutionError::Environment(err)
    }
}

impl From<EnvironmentError> for InputError {
    fn from(value: EnvironmentError) -> Self {
        let (hint, sp) = match &value {
            EnvironmentError::Other { .. } => (
                "Check variable names and scope declarations",
                lykiadb_common::error::Span::default(),
            ),
        };

        InputError::new(&value.to_string(), hint, Some(sp))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{global::GLOBAL_INTERNER, value::RV};

    #[test]
    fn test_read_basic() {
        let env_man = super::EnvironmentFrame::new(None);
        env_man.define(GLOBAL_INTERNER.intern("five"), RV::Double(5.0));
        assert_eq!(
            env_man
                .read("five", &GLOBAL_INTERNER.intern("five"))
                .unwrap(),
            RV::Double(5.0)
        );
    }

    #[test]
    fn test_read_from_parent() {
        let root = super::EnvironmentFrame::new(None);
        root.define(GLOBAL_INTERNER.intern("five"), RV::Double(5.0));
        let child = super::EnvironmentFrame::new(Some(Arc::new(root)));
        assert_eq!(
            child.read("five", &GLOBAL_INTERNER.intern("five")).unwrap(),
            RV::Double(5.0)
        );
    }

    #[test]
    fn test_write_to_parent() {
        let root = Arc::new(super::EnvironmentFrame::new(None));

        root.define(GLOBAL_INTERNER.intern("five"), RV::Double(5.0));

        let child = super::EnvironmentFrame::new(Some(root.clone()));

        child
            .assign("five", GLOBAL_INTERNER.intern("five"), RV::Double(5.1))
            .unwrap();

        assert_eq!(
            root.read("five", &GLOBAL_INTERNER.intern("five")).unwrap(),
            RV::Double(5.1)
        );

        assert_eq!(
            child.read("five", &GLOBAL_INTERNER.intern("five")).unwrap(),
            RV::Double(5.1)
        );
    }

    #[test]
    fn test_read_undefined_variable() {
        let env = super::EnvironmentFrame::new(None);
        assert!(env.read("five", &GLOBAL_INTERNER.intern("five")).is_err());
    }

    #[test]
    fn test_assign_to_undefined_variable() {
        let env = super::EnvironmentFrame::new(None);
        assert!(
            env.assign("five", GLOBAL_INTERNER.intern("five"), RV::Double(5.0))
                .is_err()
        );
    }
}
