use std::{
    collections::{BTreeMap, btree_map::Range},
    ops::RangeFull,
};

use crate::store::{Store, error::StoreError};

pub struct MemoryStore {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore {
            data: BTreeMap::new(),
        }
    }
}

pub struct MemoryScanIterator<'a>(Range<'a, Vec<u8>, Vec<u8>>);

impl Iterator for MemoryScanIterator<'_> {
    type Item = Result<(Vec<u8>, Vec<u8>), StoreError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| Ok((k.clone(), v.clone())))
    }
}

impl<'a> Store<'a> for MemoryStore {
    type ScanIterator = MemoryScanIterator<'a>;

    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) {
        self.data.insert(key.to_vec(), value);
    }

    fn delete(&mut self, key: &[u8]) {
        self.data.remove(key);
    }

    fn scan(&'a self) -> Self::ScanIterator {
        MemoryScanIterator(self.data.range::<Vec<u8>, RangeFull>(..))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> MemoryStore {
        MemoryStore::new()
    }

    #[test]
    fn test_set_and_get() {
        let mut store = make_store();
        store.set(b"key", b"value".to_vec());
        assert_eq!(store.get(b"key"), Some(b"value".to_vec()));
    }

    #[test]
    fn test_overwrite_existing_key() {
        let mut store = make_store();
        store.set(b"key", b"first".to_vec());
        store.set(b"key", b"second".to_vec());
        assert_eq!(store.get(b"key"), Some(b"second".to_vec()));
    }

    #[test]
    fn test_delete_existing_key() {
        let mut store = make_store();
        store.set(b"key", b"value".to_vec());
        store.delete(b"key");
        assert_eq!(store.get(b"key"), None);
    }

    #[test]
    fn test_delete_is_idempotent() {
        let mut store = make_store();
        // deleting a key that was never inserted should not panic
        store.delete(b"ghost");
        assert_eq!(store.get(b"ghost"), None);
    }

    #[test]
    fn test_get_absent_key_returns_none() {
        let store = make_store();
        assert_eq!(store.get(b"missing"), None);
    }

    #[test]
    fn test_get_after_delete_returns_none() {
        let mut store = make_store();
        store.set(b"key", b"value".to_vec());
        store.delete(b"key");
        assert_eq!(store.get(b"key"), None);
    }

    #[test]
    fn test_scan_empty_store() {
        let store = make_store();
        let entries: Vec<_> = store.scan().collect();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_scan_returns_all_entries_sorted() {
        let mut store = make_store();
        store.set(b"c", b"3".to_vec());
        store.set(b"a", b"1".to_vec());
        store.set(b"b", b"2".to_vec());

        let entries: Vec<_> = store.scan().collect::<Result<_, _>>().unwrap();
        assert_eq!(
            entries,
            vec![
                (b"a".to_vec(), b"1".to_vec()),
                (b"b".to_vec(), b"2".to_vec()),
                (b"c".to_vec(), b"3".to_vec()),
            ]
        );
    }

    #[test]
    fn test_scan_excludes_deleted_entries() {
        let mut store = make_store();
        store.set(b"keep", b"yes".to_vec());
        store.set(b"drop", b"no".to_vec());
        store.delete(b"drop");

        let entries: Vec<_> = store.scan().collect::<Result<_, _>>().unwrap();
        assert_eq!(entries, vec![(b"keep".to_vec(), b"yes".to_vec())]);
    }
}
