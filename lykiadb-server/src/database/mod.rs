pub mod error;
pub mod transaction;

use std::borrow::Cow;
use bson::oid::ObjectId;

use crate::{
    database::error::DatabaseError, execution::error::ExecutionError, storage::{StorageEngine, memory::MemoryStorageEngine}, value::RV
};

pub type StorageIteratorItem<'a> = Result<(Vec<u8>, RV<'a>), error::DatabaseError>;

pub enum Key<'a> {
    Collection(Cow<'a, str>),
    Document(Cow<'a, str>, Cow<'a, ObjectId>),
}

impl<'a> Key<'a> {
    fn encode(&self) -> Vec<u8> {
        match self {
            Key::Collection(name) => name.as_bytes().to_vec(),
            Key::Document(coll, id) => {
                let mut k = coll.as_bytes().to_vec();
                k.push(b':');
                k.extend_from_slice(id.to_string().as_bytes());
                k
            }
        }
    }
}

pub struct Database<S: for<'a> StorageEngine<'a>> {
    storage: S,
}

impl Default for Database<MemoryStorageEngine> {
    fn default() -> Self {
        Self::new()
    }
}

impl Database<MemoryStorageEngine> {
    pub fn new() -> Self {
        Database {
            storage: MemoryStorageEngine::new(),
        }
    }
}

impl<S: for<'a> StorageEngine<'a>> Database<S> {
    pub fn get(&'_ self, key: Key<'_>) -> Option<RV<'_>> {
        let encoded_key = key.encode();
        self.storage
            .get(&encoded_key)
            .map(|value| bson::deserialize_from_slice(&value).unwrap())
    }

    pub fn set(&mut self, key: Key<'_>, value: RV<'_>) -> Result<(), ExecutionError> {
        if !value.is_object() {
            return Err(ExecutionError::Storage(DatabaseError::InvalidValue));
        }

        let encoded_key = key.encode();
        self.storage
            .set(&encoded_key, bson::serialize_to_vec(&value).unwrap());

        Ok(())
    }

    pub fn delete(&mut self, key: Key<'_>) {
        let encoded_key = key.encode();
        self.storage.delete(&encoded_key);
    }

    pub fn scan(
        &self,
        prefix: Key<'_>,
    ) -> impl Iterator<Item = StorageIteratorItem<'_>> + '_ {
        let prefix: Vec<u8> = prefix.encode();
        self.storage.scan().filter(move |res| {
            if let Ok((k, _)) = res {
                k.starts_with(&prefix)
            } else {
                false
            }
        })
        .map(|res| {
            res.map(|(k, v)| {
                let value: RV = bson::deserialize_from_slice(&v).unwrap();
                Ok((k, value))
            }).unwrap_or_else(|e| Err(DatabaseError::Engine(e)))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, sync::Arc};

    use bson::oid::ObjectId;

    use super::*;
    use crate::{
        execution::error::ExecutionError, value::object::RVObject,
    };

    fn make_database() -> Database<MemoryStorageEngine> {
        Database::new()
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
        let database = make_database();
        let id = ObjectId::new();
        assert!(database
            .get(Key::Document(Cow::Borrowed("ns"), Cow::Owned(id)))
            .is_none());
    }

    #[test]
    fn test_set_and_get_returns_object_with_correct_fields() {
        let mut database = make_database();
        let id = ObjectId::new();
        database
            .set(
                Key::Document(Cow::Borrowed("ns"), Cow::Owned(id)),
                make_object(&[("x", RV::Int32(42))]),
            )
            .unwrap();
        let rv = database
            .get(Key::Document(Cow::Borrowed("ns"), Cow::Owned(id)))
            .unwrap();
        assert!(rv.is_object());
        let obj = rv.extract_object().unwrap();
        assert!(matches!(obj.get("x"), Some(RV::Int32(42))));
    }

    #[test]
    fn test_set_non_object_returns_invalid_value_error() {
        let mut database = make_database();
        let id = ObjectId::new();
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
                database.set(
                    Key::Document(Cow::Borrowed("ns"), Cow::Owned(id)),
                    value.clone()
                ),
                Err(ExecutionError::Storage(DatabaseError::InvalidValue))
            );
        }
    }

    #[test]
    fn test_delete_removes_document() {
        let mut database = make_database();
        let id = ObjectId::new();
        database
            .set(
                Key::Document(Cow::Borrowed("ns"), Cow::Owned(id)),
                make_object(&[("flag", RV::Bool(true))]),
            )
            .unwrap();
        assert!(database
            .get(Key::Document(Cow::Borrowed("ns"), Cow::Owned(id)))
            .is_some());
        database.delete(Key::Document(Cow::Borrowed("ns"), Cow::Owned(id)));
        assert!(database
            .get(Key::Document(Cow::Borrowed("ns"), Cow::Owned(id)))
            .is_none());
    }

    #[test]
    fn test_scan_returns_docs_in_sorted_key_order() {
        let mut database = make_database();
        // Use fixed bytes so the hex-encoded sort order is deterministic
        let id1 = ObjectId::from_bytes([0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let id2 = ObjectId::from_bytes([0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let id3 = ObjectId::from_bytes([0x03, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        // Insert in non-sorted order; BTreeMap will return them sorted by encoded key
        for id in [id3, id1, id2] {
            database
                .set(
                    Key::Document(Cow::Borrowed("ns"), Cow::Owned(id)),
                    make_object(&[("k", RV::Str(id.to_string().into()))]),
                )
                .unwrap();
        }
        let prefix_len = "ns:".len();
        let ids_returned: Vec<ObjectId> = database
            .scan(Key::Collection(Cow::Borrowed("ns")))
            .map(|r| {
                let (k, v) = r.unwrap();
                let id_str = String::from_utf8(k[prefix_len..].to_vec()).unwrap();
                assert_eq!(v.extract_object().unwrap().get("k"), Some(RV::Str(id_str.clone().into())));
                ObjectId::parse_str(&id_str).unwrap()
            })
            .collect();
        assert_eq!(ids_returned, vec![id1, id2, id3]);
    }

    #[test]
    fn test_scan_does_not_return_other_namespace_docs() {
        let mut database = make_database();
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();
        let id3 = ObjectId::new();
        database
            .set(
                Key::Document(Cow::Borrowed("ns1"), Cow::Owned(id1)),
                make_object(&[("id", RV::Int32(1))]),
            )
            .unwrap();
        database
            .set(
                Key::Document(Cow::Borrowed("ns1"), Cow::Owned(id2)),
                make_object(&[("id", RV::Int32(2))]),
            )
            .unwrap();
        database
            .set(
                Key::Document(Cow::Borrowed("ns2"), Cow::Owned(id3)),
                make_object(&[("id", RV::Int32(3))]),
            )
            .unwrap();
        assert_eq!(
            database.scan(Key::Collection(Cow::Borrowed("ns1"))).count(),
            2
        );
        assert_eq!(
            database.scan(Key::Collection(Cow::Borrowed("ns2"))).count(),
            1
        );
    }
}
