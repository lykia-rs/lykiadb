pub mod error;

use std::iter::Filter;

use crate::{engine::error::EngineError, execution::error::ExecutionError, store::{IteratorItem, Store, memory::{MemoryScanIterator, MemoryStore}}, value::RV};

pub struct StoreId(String);

pub struct Catalog<S: for<'a> Store<'a>> {
    store: S
}

pub struct Engine<S: for<'a> Store<'a>> {
    catalog: Catalog<S>
}

fn encode_key(sid: &StoreId, key: &str) -> Vec<u8> {
    let mut k = sid.0.clone();
    k.push_str(key);
    k.into_bytes()
}

impl Engine<MemoryStore> {
    pub fn new() -> Self {
        Engine {
            catalog: Catalog {
                store: MemoryStore::new(),
            }
        }
    }
}

impl Engine<MemoryStore> {
    pub fn get(&'_ self, sid: &StoreId, key: &str) -> Option<RV<'_>> {
        let encoded_key = encode_key(sid, key);
        self.catalog.store.get(&encoded_key).map(|value| bson::deserialize_from_slice(&value).unwrap())
    }
    pub fn set(&mut self, sid: &StoreId, key: &str, value: RV<'_>) -> Result<(), ExecutionError> {

        if !value.is_object() {
            return Err(ExecutionError::Engine(EngineError::InvalidValue));
        }

        let encoded_key = encode_key(sid, key);
        self.catalog.store.set(&encoded_key, bson::serialize_to_vec(&value).unwrap());
        
        Ok(())
    }
    pub fn delete(&mut self, sid: &StoreId, key: &str) {
        let encoded_key = encode_key(sid, key);
        self.catalog.store.delete(&encoded_key);
    }
    pub fn scan(&'_ self, sid: &StoreId) -> Filter<MemoryScanIterator<'_>, impl FnMut(&IteratorItem) -> bool + '_> {
        let prefix: Vec<u8> = sid.0.clone().into_bytes();
        self.catalog.store.scan().filter(move |res| {
            if let Ok((k, _)) = res {
                k.starts_with(&prefix)
            } else {
                false
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        engine::error::EngineError,
        execution::error::ExecutionError,
        value::object::RVObject,
    };

    fn make_engine() -> Engine<MemoryStore> {
        Engine::new()
    }

    fn make_sid(name: &str) -> StoreId {
        StoreId(name.to_string())
    }

    fn make_object(fields: &[(&str, RV<'static>)]) -> RV<'static> {
        let mut obj = RVObject::new();
        for (key, value) in fields {
            obj.insert(key.to_string(), value.clone());
        }
        RV::Object(obj)
    }

    #[test]
    fn test_get_missing_key_returns_none() {
        let engine = make_engine();
        let sid = make_sid("ns:");
        assert!(engine.get(&sid, "nonexistent").is_none());
    }

    #[test]
    fn test_set_and_get_returns_object_with_correct_fields() {
        let mut engine = make_engine();
        let sid = make_sid("ns:");
        engine
            .set(&sid, "doc1", make_object(&[("x", RV::Int32(42))]))
            .unwrap();
        let rv = engine.get(&sid, "doc1").unwrap();
        assert!(rv.is_object());
        let obj = rv.extract_object().unwrap();
        assert!(matches!(obj.get("x"), Some(RV::Int32(42))));
    }

    #[test]
    fn test_set_non_object_returns_invalid_value_error() {
        let mut engine = make_engine();
        let sid = make_sid("ns:");
        let non_objects: &[RV<'static>] = &[
            RV::Null,
            RV::Bool(false),
            RV::Int32(1),
            RV::Int64(2),
            RV::Double(3.0),
            RV::Str(Arc::new("text".to_string())),
        ];
        for value in non_objects {
            assert_eq!(
                engine.set(&sid, "doc", value.clone()),
                Err(ExecutionError::Engine(EngineError::InvalidValue))
            );
        }
    }

    #[test]
    fn test_delete_removes_document() {
        let mut engine = make_engine();
        let sid = make_sid("ns:");
        engine
            .set(&sid, "doc1", make_object(&[("flag", RV::Bool(true))]))
            .unwrap();
        assert!(engine.get(&sid, "doc1").is_some());
        engine.delete(&sid, "doc1");
        assert!(engine.get(&sid, "doc1").is_none());
    }

    #[test]
    fn test_scan_returns_docs_in_sorted_key_order() {
        let mut engine = make_engine();
        let sid = make_sid("ns:");
        // Insert in non-alphabetical order; BTreeMap will return them sorted
        for key in ["gamma", "alpha", "beta"] {
            engine
                .set(&sid, key, make_object(&[("k", RV::Str(Arc::new(key.to_string())))]))
                .unwrap();
        }
        let prefix_len = sid.0.len();
        let keys: Vec<String> = engine
            .scan(&sid)
            .map(|r| {
                let (k, _) = r.unwrap();
                String::from_utf8(k[prefix_len..].to_vec()).unwrap()
            })
            .collect();
        assert_eq!(keys, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_scan_does_not_return_other_namespace_docs() {
        let mut engine = make_engine();
        let sid1 = make_sid("ns1:");
        let sid2 = make_sid("ns2:");
        engine
            .set(&sid1, "a", make_object(&[("id", RV::Int32(1))]))
            .unwrap();
        engine
            .set(&sid1, "b", make_object(&[("id", RV::Int32(2))]))
            .unwrap();
        engine
            .set(&sid2, "c", make_object(&[("id", RV::Int32(3))]))
            .unwrap();
        assert_eq!(engine.scan(&sid1).count(), 2);
        assert_eq!(engine.scan(&sid2).count(), 1);
    }
}