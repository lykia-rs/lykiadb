use rustc_hash::FxHashMap;

use crate::{
    util::{Shared, alloc_shared},
    value::RV,
};

#[derive(Debug, Clone)]
pub struct RVObject {
    inner: Shared<FxHashMap<String, RV>>,
}

impl Default for RVObject {
    fn default() -> Self {
        Self::new()
    }
}

impl RVObject {
    pub fn new() -> Self {
        RVObject {
            inner: alloc_shared(FxHashMap::default()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    pub fn get(&self, key: &str) -> Option<RV> {
        let r = self.inner.read().unwrap();
        if !r.contains_key(key) {
            return None;
        }
        let cloned = r.get(key).unwrap().clone();
        Some(cloned)
    }

    pub fn insert(&mut self, key: String, value: RV) {
        self.inner.write().unwrap().insert(key, value);
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.read().unwrap().contains_key(key)
    }

    pub fn keys(&self) -> Box<dyn Iterator<Item = String> + '_> {
        let keys = self
            .inner
            .read()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        Box::new(keys.into_iter())
    }

    pub fn from_map(map: FxHashMap<String, RV>) -> Self {
        RVObject {
            inner: alloc_shared(map),
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (String, RV)> + '_> {
        let items = self
            .inner
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>();
        Box::new(items.into_iter())
    }
}
