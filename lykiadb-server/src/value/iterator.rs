use dyn_clone::DynClone;
use interb::Symbol;
use rustc_hash::FxHashMap;

use crate::value::RV;

pub type RVs = Box<dyn RVIterator>;

pub trait RVIterator: Iterator<Item = IterationEnvironment> + DynClone {}

dyn_clone::clone_trait_object!(RVIterator);

impl<I: Iterator<Item = IterationEnvironment> + DynClone> RVIterator for I {}

#[derive(Debug, Clone)]
pub struct IterationEnvironment {
    pub inner: FxHashMap<Symbol, RV>,
}

impl IterationEnvironment {
    pub fn new(keys: Vec<Symbol>, values: Vec<RV>) -> Self {
        let mut inner = FxHashMap::default();
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            inner.insert(key, value);
        }
        IterationEnvironment { inner }
    }

    pub fn get(&self, key: &Symbol) -> Option<&RV> {
        self.inner.get(key)
    }

    pub fn spread_to(&self, target: &mut FxHashMap<Symbol, RV>) {
        for (key, value) in &self.inner {
            target.insert(key.clone(), value.clone());
        }
    }
}