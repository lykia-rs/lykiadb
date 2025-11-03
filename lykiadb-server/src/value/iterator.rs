use dyn_clone::DynClone;
use interb::Symbol;
use smallvec::SmallVec;

use crate::value::RV;

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
}