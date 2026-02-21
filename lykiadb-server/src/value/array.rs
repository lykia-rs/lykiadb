use crate::{
    util::{Shared, alloc_shared},
    value::RV,
};

#[derive(Debug, Clone)]
pub struct RVArray<'arena> {
    inner: Shared<Vec<RV<'arena>>>,
}

impl<'arena> Default for RVArray<'arena> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'arena> RVArray<'arena> {
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

    pub fn get(&self, index: usize) -> RV<'arena> {
        self.inner.read().unwrap()[index].clone()
    }

    pub fn contains(&self, value: &RV<'arena>) -> bool {
        self.inner.read().unwrap().contains(value)
    }

    pub fn insert(&mut self, value: RV<'arena>) {
        self.inner.write().unwrap().push(value);
    }

    pub fn from_vec(vec: Vec<RV<'arena>>) -> Self {
        RVArray {
            inner: alloc_shared(vec),
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = RV<'arena>> + '_> {
        let items = self
            .inner
            .read()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        Box::new(items.into_iter())
    }

    pub fn collect(&self) -> Vec<RV<'arena>> {
        self.inner.read().unwrap().iter().cloned().collect()
    }
}
