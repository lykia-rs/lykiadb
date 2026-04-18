use std::collections::BTreeMap;

use crate::store::{ScanIterator, Store};

pub struct MemoryStore {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore {
            data: BTreeMap::new(),
        }
    }
}

impl<'a> Store<'a> for MemoryStore {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) {
        self.data.insert(key.to_vec(), value);
    }

    fn delete(&mut self, key: &[u8]) {
        self.data.remove(key);
    }

    fn scan(&'a self) -> ScanIterator<'a> {
        Box::new(
            self.data
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
        )
    }

    fn scan_prefix(&'a self, prefix: &'a [u8]) -> ScanIterator<'a> {
        Box::new(
            self.data
                .iter()
                .filter(move |(k, _)| k.starts_with(prefix))
                .map(|(k, v)| (k.clone(), v.clone()))
        )
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

        let entries: Vec<_> = store.scan().collect();
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

        let entries: Vec<_> = store.scan().collect();
        assert_eq!(entries, vec![(b"keep".to_vec(), b"yes".to_vec())]);
    }

    #[test]
    fn test_scan_prefix_returns_matching_entries() {
        let mut store = make_store();
        store.set(b"col:1", b"alice".to_vec());
        store.set(b"col:2", b"bob".to_vec());
        store.set(b"idx:1", b"should_not_appear".to_vec());

        let entries: Vec<_> = store.scan_prefix(b"col:").collect();
        assert_eq!(
            entries,
            vec![
                (b"col:1".to_vec(), b"alice".to_vec()),
                (b"col:2".to_vec(), b"bob".to_vec()),
            ]
        );
    }

    #[test]
    fn test_scan_prefix_empty_when_no_match() {
        let mut store = make_store();
        store.set(b"col:1", b"alice".to_vec());

        let entries: Vec<_> = store.scan_prefix(b"idx:").collect();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_scan_prefix_excludes_deleted_entries() {
        let mut store = make_store();
        store.set(b"col:1", b"alice".to_vec());
        store.set(b"col:2", b"bob".to_vec());
        store.delete(b"col:1");

        let entries: Vec<_> = store.scan_prefix(b"col:").collect();
        assert_eq!(entries, vec![(b"col:2".to_vec(), b"bob".to_vec())]);
    }

    #[test]
    fn test_scan_prefix_longer_than_key_returns_nothing() {
        let mut store = make_store();
        store.set(b"col:100", b"val".to_vec());

        let entries: Vec<_> = store.scan_prefix(b"col:100:extra").collect();
        assert!(entries.is_empty());
    }
}