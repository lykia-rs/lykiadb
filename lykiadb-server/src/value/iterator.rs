use rustc_hash::FxHashMap;

use crate::value::RV;

pub type RVIterator = Box<dyn Iterator<Item = IterationEnvironment>>;

#[derive(Debug, Clone)]
pub struct IterationEnvironment {
    pub inner: FxHashMap<String, RV>,
}

impl IterationEnvironment {
    pub fn new(keys: Vec<String>, values: Vec<RV>) -> Self {
        let mut inner = FxHashMap::default();
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            inner.insert(key, value);
        }
        IterationEnvironment { inner }
    }

    pub fn get(&self, key: &str) -> Option<&RV> {
        self.inner.get(key)
    }

    pub fn spread_to(&self, target: &mut FxHashMap<String, RV>) {
        for (key, value) in &self.inner {
            target.insert(key.clone(), value.clone());
        }
    }
}