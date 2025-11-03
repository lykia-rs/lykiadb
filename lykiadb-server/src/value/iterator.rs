use dyn_clone::DynClone;
use interb::Symbol;
use rustc_hash::FxHashMap;

use crate::value::RV;

pub type RVs = Box<dyn RVIterator>;

pub trait RVIterator: Iterator<Item = IterationEnvironment> + DynClone {}

dyn_clone::clone_trait_object!(RVIterator);

impl<I: Iterator<Item = IterationEnvironment> + DynClone> RVIterator for I {}
/*
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
}*/

// IterationEnvironment but with key index based matching without a map
#[derive(Debug, Clone)]
pub struct IterationEnvironment {
    pub keys: Vec<Symbol>,
    pub values: Vec<RV>,
}

impl IterationEnvironment {
    pub fn new(keys: Vec<Symbol>, values: Vec<RV>) -> Self {
        IterationEnvironment { keys, values }
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