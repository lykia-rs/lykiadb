use bytes::BufMut;

use crate::block::builder::{DataKeyLen, SIZEOF_DATA_KEY_LEN};

type Key = Vec<u8>;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MetaKeyRange {
    has_keys: bool,
    pub(crate) min_key: Key,
    pub(crate) max_key: Key,
}

impl Default for MetaKeyRange {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaKeyRange {
    pub fn new() -> Self {
        MetaKeyRange {
            has_keys: false,
            min_key: Vec::new(),
            max_key: Vec::new(),
        }
    }

    pub fn build(min_key: Key, max_key: Key) -> Self {
        let mut range = MetaKeyRange::new();
        range.add(&min_key);
        range.add(&max_key);
        range
    }

    pub fn add(&mut self, key: &[u8]) {
        if !self.has_keys || key < self.min_key.as_slice() {
            self.min_key = key.to_vec();
        }
        if !self.has_keys || key > self.max_key.as_slice() {
            self.max_key = key.to_vec();
        }
        self.has_keys = true;
    }

    // ------------------------------------------------------------------------------------
    // |   key len (u16)   |   min_key (N u8)   |    key len (u16)   |   max_key (N u8)   |
    // ------------------------------------------------------------------------------------
    pub fn write_to(&self, buffer: &mut Vec<u8>) {
        if self.has_keys {
            buffer.put_u16(self.min_key.len() as DataKeyLen);
            buffer.extend_from_slice(&self.min_key);
            buffer.put_u16(self.max_key.len() as DataKeyLen);
            buffer.extend_from_slice(&self.max_key);
        }
    }

    pub fn len(&self) -> usize {
        if self.has_keys {
            self.min_key.len() + self.max_key.len() + 2 * SIZEOF_DATA_KEY_LEN
        } else {
            0
        }
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mut merged = MetaKeyRange::new();
        if self.has_keys || other.has_keys {
            merged.add(&self.min_key);
            merged.add(&self.max_key);
            merged.add(&other.min_key);
            merged.add(&other.max_key);
        }
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_range() {
        let krange = MetaKeyRange::new();
        assert!(krange.min_key.is_empty());
        assert!(krange.max_key.is_empty());
        assert_eq!(krange.len(), 0);
    }

    #[test]
    fn test_add_single_key() {
        let mut krange = MetaKeyRange::new();
        let key = b"test_key";

        krange.add(key);

        assert_eq!(krange.min_key, key);
        assert_eq!(krange.max_key, key);
        assert_eq!(krange.len(), key.len() * 2 + 4); // 8 + 8 + 4 = 20
    }

    #[test]
    fn test_add_multiple_keys_ascending() {
        let mut krange = MetaKeyRange::new();
        let keys = [b"a", b"b", b"c", b"d"];

        for key in &keys {
            krange.add(*key);
        }

        assert_eq!(krange.min_key, b"a");
        assert_eq!(krange.max_key, b"d");
        assert_eq!(krange.len(), 6); // 1 + 1 + 4 = 6
    }

    #[test]
    fn test_add_multiple_keys_descending() {
        let mut krange = MetaKeyRange::new();
        let keys = [b"d", b"c", b"b", b"a"];

        for key in &keys {
            krange.add(*key);
        }

        assert_eq!(krange.min_key, b"a");
        assert_eq!(krange.max_key, b"d");
        assert_eq!(krange.len(), 6); // 1 + 1 + 4 = 6
    }

    #[test]
    fn test_add_multiple_keys_random_order() {
        let mut krange = MetaKeyRange::new();
        let keys: &[&[u8]] = &[b"zebra", b"apple", b"dog", b"cat"];

        for key in keys {
            krange.add(key);
        }

        assert_eq!(krange.min_key, b"apple");
        assert_eq!(krange.max_key, b"zebra");
        assert_eq!(krange.len(), 14); // 5 + 5 + 4 = 14
    }

    #[test]
    fn test_add_duplicate_keys() {
        let mut krange = MetaKeyRange::new();
        let key = b"duplicate";

        krange.add(key);
        krange.add(key);
        krange.add(key);

        assert_eq!(krange.min_key, key);
        assert_eq!(krange.max_key, key);
        assert_eq!(krange.len(), key.len() * 2 + 4); // 9 + 9 + 4 = 22
    }

    #[test]
    fn test_add_empty_key() {
        let mut krange = MetaKeyRange::new();
        let empty_key = b"";
        let non_empty_key = b"test";

        krange.add(empty_key);
        krange.add(non_empty_key);

        assert_eq!(krange.min_key, empty_key);
        assert_eq!(krange.max_key, non_empty_key);
        assert_eq!(krange.len(), 8); // 0 + 4 + 4 = 8
    }
    #[test]
    fn test_add_keys_with_different_lengths() {
        let mut krange = MetaKeyRange::new();
        let keys: &[&[u8]] = &[b"a", b"longer_key", b"mid"];

        for key in keys {
            krange.add(key);
        }

        assert_eq!(krange.min_key, b"a");
        assert_eq!(krange.max_key, b"mid");
        assert_eq!(krange.len(), 8); // 1 + 3 + 4 = 8
    }

    #[test]
    fn test_add_binary_keys() {
        let mut krange = MetaKeyRange::new();
        let key1 = &[0x00, 0x01, 0x02];
        let key2 = &[0xFF, 0xFE, 0xFD];
        let key3 = &[0x80, 0x40, 0x20];

        krange.add(key1);
        krange.add(key2);
        krange.add(key3);

        assert_eq!(krange.min_key, key1);
        assert_eq!(krange.max_key, key2);
        assert_eq!(krange.len(), 10); // 3 + 3 + 4 = 10
    }

    #[test]
    fn test_finish_empty_range() {
        let krange = MetaKeyRange::new();
        let mut buffer = Vec::new();

        krange.write_to(&mut buffer);

        assert!(buffer.is_empty());
    }

    #[test]
    fn test_finish_with_keys() {
        let mut krange = MetaKeyRange::new();
        krange.add(b"max");
        krange.add(b"min");

        let mut buffer = Vec::new();
        krange.write_to(&mut buffer);

        let mut expected = Vec::new();
        expected.extend_from_slice(&(3u16).to_be_bytes()); // min_key length
        expected.extend_from_slice(b"max"); // min_key is "max" 
        expected.extend_from_slice(&(3u16).to_be_bytes()); // max_key length
        expected.extend_from_slice(b"min"); // max_key is "min"
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_finish_appends_to_existing_buffer() {
        let mut krange = MetaKeyRange::new();
        krange.add(b"a");
        krange.add(b"z");

        let mut buffer = vec![1, 2, 3];
        krange.write_to(&mut buffer);

        let mut expected = vec![1, 2, 3];
        expected.extend_from_slice(&(1u16).to_be_bytes()); // min_key length
        expected.extend_from_slice(b"a"); // min_key
        expected.extend_from_slice(&(1u16).to_be_bytes()); // max_key length  
        expected.extend_from_slice(b"z"); // max_key
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_size_consistency() {
        let mut krange = MetaKeyRange::new();
        assert_eq!(krange.len(), 0);

        krange.add(b"test");
        assert_eq!(krange.len(), 12); // "test" appears as both min and max: 4 + 4 + 4 = 12

        krange.add(b"another");
        assert_eq!(krange.len(), 15); // "another" (7) + "test" (4) + 4 = 15

        krange.add(b"key");
        assert_eq!(krange.len(), 15); // "another" (7) + "test" (4) + 4 = 15 - no change
    }

    #[test]
    fn test_lexicographic_ordering() {
        let mut krange = MetaKeyRange::new();

        // These should be ordered lexicographically, not by length
        krange.add(b"b");
        krange.add(b"aa");
        krange.add(b"c");

        assert_eq!(krange.min_key, b"aa");
        assert_eq!(krange.max_key, b"c");
    }

    #[test]
    fn test_case_sensitivity() {
        let mut krange = MetaKeyRange::new();

        krange.add(b"A");
        krange.add(b"a");
        krange.add(b"B");
        krange.add(b"b");

        // ASCII values: A=65, B=66, a=97, b=98
        assert_eq!(krange.min_key, b"A");
        assert_eq!(krange.max_key, b"b");
    }
}
