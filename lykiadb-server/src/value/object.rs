use indexmap::IndexMap;
use lykiadb_common::memory::{Shared, alloc_shared};
use serde::de::MapAccess;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::value::RV;

#[derive(Debug, Clone)]
pub struct RVObject<'v> {
    inner: Shared<IndexMap<String, RV<'v>>>,
}

impl<'v> Default for RVObject<'v> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'v> RVObject<'v> {
    pub fn new() -> Self {
        RVObject {
            inner: alloc_shared(IndexMap::default()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    pub fn get(&self, key: &str) -> Option<RV<'v>> {
        let r = self.inner.read().unwrap();
        if !r.contains_key(key) {
            return None;
        }
        let cloned = r.get(key).unwrap().clone();
        Some(cloned)
    }

    pub fn insert(&mut self, key: String, value: RV<'v>) {
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

    pub fn from_map(map: IndexMap<String, RV<'v>>) -> Self {
        RVObject {
            inner: alloc_shared(map),
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (String, RV<'v>)> + '_> {
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

impl<'v> Serialize for RVObject<'v> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let map = self.inner.read().unwrap();
        let mut s = serializer.serialize_map(Some(map.len()))?;
        for (k, v) in map.iter() {
            s.serialize_entry(k, v)?;
        }
        s.end()
    }
}

struct RVObjectVisitor<'v> {
    _phantom: std::marker::PhantomData<&'v ()>,
}

impl<'de, 'v> serde::de::Visitor<'de> for RVObjectVisitor<'v> {
    type Value = RVObject<'v>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map of RV values")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut obj_map = IndexMap::default();
        while let Some((key, value)) = map.next_entry::<String, RV>()? {
            obj_map.insert(key, value);
        }
        Ok(RVObject::from_map(obj_map))
    }
}

impl<'de, 'v> Deserialize<'de> for RVObject<'v> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RVObjectVisitor {
            _phantom: std::marker::PhantomData,
        })
    }
}
