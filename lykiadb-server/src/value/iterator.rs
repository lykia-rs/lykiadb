use dyn_clone::DynClone;
use interb::Symbol;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{global::GLOBAL_INTERNER, value::{RV, object::RVObject}};

pub type RVs = Box<dyn RVIterator>;

pub trait RVIterator: Iterator<Item = ExecutionRow> + DynClone {}

dyn_clone::clone_trait_object!(RVIterator);

impl<I: Iterator<Item = ExecutionRow> + DynClone> RVIterator for I {}

#[derive(Debug, Clone)]
pub struct ExecutionRow {
    pub keys: SmallVec<[Symbol; 4]>,
    pub values: SmallVec<[RV; 4]>
}

impl ExecutionRow {
    pub fn new() -> Self {
        ExecutionRow { keys: SmallVec::new(), values: SmallVec::new() }
    }

    pub fn get(&self, key: &Symbol) -> Option<&RV> {
        if let Some(pos) = self.keys.iter().position(|k| k == key) {
            self.values.get(pos)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: Symbol, value: RV) {
        self.keys.push(key);
        self.values.push(value);
    }

    pub fn as_value(&self) -> RV {
        let mut map = RVObject::new();
        for (k, v) in self.keys.iter().zip(self.values.iter()) {
            map.insert(GLOBAL_INTERNER.resolve(*k).unwrap().to_string(), v.clone());
        }
        RV::Object(map)
    }
}