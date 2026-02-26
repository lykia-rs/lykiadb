use lykiadb_common::memory::{Shared, alloc_shared};

use crate::value::RV;

#[derive(Debug, Clone)]
pub struct RVArray<'v> {
    inner: Shared<Vec<RV<'v>>>,
}

impl<'v> Default for RVArray<'v> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'v> RVArray<'v> {
    pub fn new() -> Self {
        RVArray {
            inner: alloc_shared(Vec::new()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    pub fn get(&self, index: usize) -> RV<'v> {
        self.inner.read().unwrap()[index].clone()
    }

    pub fn contains(&self, value: &RV<'v>) -> bool {
        self.inner.read().unwrap().contains(value)
    }

    pub fn insert(&mut self, value: RV<'v>) {
        self.inner.write().unwrap().push(value);
    }

    pub fn from_vec(vec: Vec<RV<'v>>) -> Self {
        RVArray {
            inner: alloc_shared(vec),
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = RV<'v>> + '_> {
        let items = self
            .inner
            .read()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        Box::new(items.into_iter())
    }

    pub fn collect(&self) -> Vec<RV<'v>> {
        self.inner.read().unwrap().iter().cloned().collect()
    }
}
