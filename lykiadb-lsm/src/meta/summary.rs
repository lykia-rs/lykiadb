pub struct MetaSummary {
    has_keys: bool,
    min_key: Vec<u8>,
    max_key: Vec<u8>,
}

impl MetaSummary {
    pub fn new() -> Self {
        MetaSummary {
            has_keys: false,
            min_key: Vec::new(),
            max_key: Vec::new(),
        }
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

    pub fn finish(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.min_key);
        buffer.extend_from_slice(&self.max_key);
    }

    pub fn size(&self) -> usize {
        self.min_key.len() + self.max_key.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_summary() {
        let summary = MetaSummary::new();
        assert!(summary.min_key.is_empty());
        assert!(summary.max_key.is_empty());
        assert_eq!(summary.size(), 0);
    }

    #[test]
    fn test_add_single_key() {
        let mut summary = MetaSummary::new();
        let key = b"test_key";

        summary.add(key);

        assert_eq!(summary.min_key, key);
        assert_eq!(summary.max_key, key);
        assert_eq!(summary.size(), key.len() * 2);
    }

    #[test]
    fn test_add_multiple_keys_ascending() {
        let mut summary = MetaSummary::new();
        let keys = [b"a", b"b", b"c", b"d"];

        for key in &keys {
            summary.add(*key);
        }

        assert_eq!(summary.min_key, b"a");
        assert_eq!(summary.max_key, b"d");
        assert_eq!(summary.size(), 2); // 1 byte each for min and max
    }

    #[test]
    fn test_add_multiple_keys_descending() {
        let mut summary = MetaSummary::new();
        let keys = [b"d", b"c", b"b", b"a"];

        for key in &keys {
            summary.add(*key);
        }

        assert_eq!(summary.min_key, b"a");
        assert_eq!(summary.max_key, b"d");
        assert_eq!(summary.size(), 2); // 1 byte each for min and max
    }
    
    #[test]
    fn test_add_multiple_keys_random_order() {
        let mut summary = MetaSummary::new();
        let keys: &[&[u8]] = &[b"zebra", b"apple", b"dog", b"cat"];
        
        for key in keys {
            summary.add(key);
        }
        
        assert_eq!(summary.min_key, b"apple");
        assert_eq!(summary.max_key, b"zebra");
        assert_eq!(summary.size(), 10); // 5 bytes each for "apple" and "zebra"
    }

    #[test]
    fn test_add_duplicate_keys() {
        let mut summary = MetaSummary::new();
        let key = b"duplicate";

        summary.add(key);
        summary.add(key);
        summary.add(key);

        assert_eq!(summary.min_key, key);
        assert_eq!(summary.max_key, key);
        assert_eq!(summary.size(), key.len() * 2);
    }

    #[test]
    fn test_add_empty_key() {
        let mut summary = MetaSummary::new();
        let empty_key = b"";
        let non_empty_key = b"test";

        summary.add(empty_key);
        summary.add(non_empty_key);

        assert_eq!(summary.min_key, empty_key);
        assert_eq!(summary.max_key, non_empty_key);
        assert_eq!(summary.size(), 4); // 0 + 4 bytes
    }
    #[test]
    fn test_add_keys_with_different_lengths() {
        let mut summary = MetaSummary::new();
        let keys: &[&[u8]] = &[b"a", b"longer_key", b"mid"];
        
        for key in keys {
            summary.add(key);
        }
        
        assert_eq!(summary.min_key, b"a");
        assert_eq!(summary.max_key, b"mid");
        assert_eq!(summary.size(), 4); // 1 + 3 bytes
    }

    #[test]
    fn test_add_binary_keys() {
        let mut summary = MetaSummary::new();
        let key1 = &[0x00, 0x01, 0x02];
        let key2 = &[0xFF, 0xFE, 0xFD];
        let key3 = &[0x80, 0x40, 0x20];

        summary.add(key1);
        summary.add(key2);
        summary.add(key3);

        assert_eq!(summary.min_key, key1);
        assert_eq!(summary.max_key, key2);
        assert_eq!(summary.size(), 6); // 3 + 3 bytes
    }

    #[test]
    fn test_finish_empty_summary() {
        let summary = MetaSummary::new();
        let mut buffer = Vec::new();

        summary.finish(&mut buffer);

        assert!(buffer.is_empty());
    }
    
    #[test]
    fn test_finish_with_keys() {
        let mut summary = MetaSummary::new();
        summary.add(b"max");
        summary.add(b"min");
        
        let mut buffer = Vec::new();
        summary.finish(&mut buffer);
        
        let mut expected = Vec::new();
        expected.extend_from_slice(b"max"); // min_key comes first
        expected.extend_from_slice(b"min"); // max_key comes second
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_finish_appends_to_existing_buffer() {
        let mut summary = MetaSummary::new();
        summary.add(b"a");
        summary.add(b"z");

        let mut buffer = vec![1, 2, 3];
        summary.finish(&mut buffer);

        let mut expected = vec![1, 2, 3];
        expected.extend_from_slice(b"a");
        expected.extend_from_slice(b"z");
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_size_consistency() {
        let mut summary = MetaSummary::new();
        assert_eq!(summary.size(), 0);

        summary.add(b"test");
        assert_eq!(summary.size(), 8); // "test" appears as both min and max

        summary.add(b"another");
        assert_eq!(summary.size(), 11); // "another" (7) + "test" (4)

        summary.add(b"key");
        assert_eq!(summary.size(), 11); // "another" (7) + "test" (4) - no change
    }

    #[test]
    fn test_lexicographic_ordering() {
        let mut summary = MetaSummary::new();

        // These should be ordered lexicographically, not by length
        summary.add(b"b");
        summary.add(b"aa");
        summary.add(b"c");

        assert_eq!(summary.min_key, b"aa");
        assert_eq!(summary.max_key, b"c");
    }

    #[test]
    fn test_case_sensitivity() {
        let mut summary = MetaSummary::new();

        summary.add(b"A");
        summary.add(b"a");
        summary.add(b"B");
        summary.add(b"b");

        // ASCII values: A=65, B=66, a=97, b=98
        assert_eq!(summary.min_key, b"A");
        assert_eq!(summary.max_key, b"b");
    }
}