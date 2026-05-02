use lykiadb_common::memory::{Shared, alloc_shared};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

impl<'v> Serialize for RVArray<'v> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let vec = self.inner.read().unwrap();
        let mut s = serializer.serialize_seq(Some(vec.len()))?;
        for item in vec.iter() {
            s.serialize_element(item)?;
        }
        s.end()
    }
}

impl<'de, 'v> Deserialize<'de> for RVArray<'v> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<RV>::deserialize(deserializer)?;
        Ok(RVArray::from_vec(vec))
    }
}
