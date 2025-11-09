use bytes::Bytes;
use crossbeam_skiplist::SkipMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

struct Memtable {
    map: Arc<SkipMap<Bytes, Bytes>>,
    id: usize,
    approximate_size: Arc<AtomicUsize>,
}

impl Memtable {
    pub fn new() -> Memtable {
        Memtable {
            map: Arc::new(SkipMap::new()),
            id: 0,
            approximate_size: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<Bytes> {
        self.map.get(key).map(|x| x.value().clone())
    }

    pub fn put(&self, key: &[u8], value: &[u8]) {
        self.map
            .insert(Bytes::copy_from_slice(key), Bytes::copy_from_slice(value));
    }
}

#[cfg(test)]
mod tests {
    use crate::memtable::Memtable;
    use bytes::Bytes;

    #[test]
    fn test_create() {
        let memtable = Memtable::new();
        assert_eq!(memtable.id, 0);
    }

    #[test]
    fn test_get() {
        let memtable = Memtable::new();
        memtable.put(b"key", b"value");
        assert_eq!(&memtable.get(b"key").unwrap()[..], b"value");
    }

    #[test]
    fn test_get_absent_0() {
        let memtable = Memtable::new();
        assert_eq!(memtable.get(b"key"), None);
    }

    #[test]
    fn test_get_absent_1() {
        let memtable = Memtable::new();
        memtable.put(b"key", b"value");
        assert_eq!(memtable.get(b"other_key"), None);
    }

    #[test]
    fn test_put_0() {
        let memtable = Memtable::new();
        memtable.put(b"key", b"value");
        assert_eq!(
            &memtable
                .map
                .get(&Bytes::copy_from_slice(b"key"))
                .unwrap()
                .value()[..],
            b"value"
        );
    }

    #[test]
    fn test_put_1() {
        let memtable = Memtable::new();
        memtable.put(b"key", b"value");
        memtable.put(b"key", b"value2");
        assert_eq!(
            &memtable
                .map
                .get(&Bytes::copy_from_slice(b"key"))
                .unwrap()
                .value()[..],
            b"value2"
        );
    }
}
