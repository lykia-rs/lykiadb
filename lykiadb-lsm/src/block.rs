use bytes::BufMut;
use crate::{meta::{MetaEntryOffset, MetaKeyRange}, DataSize};

pub struct Block {
    max_size: usize,
    buffer: Vec<u8>,
    offsets: MetaEntryOffset,
    pub key_range: MetaKeyRange,
}

impl Block {
    pub fn new(max_size: usize) -> Self {
        Block {
            max_size,
            buffer: Vec::new(),
            offsets: MetaEntryOffset::new(None),
            key_range: MetaKeyRange::new(),
        }
    }

    pub fn add(&mut self, key: &[u8], value: &[u8]) -> bool {
        let key_len = key.len() as DataSize;
        let value_len = value.len() as DataSize;
        let required_for_data = key_len as usize + value_len as usize + 4;
        let required_for_meta = 2; // Size of new offset
        if required_for_data + required_for_meta + self.len() <= self.max_size {
            self.offsets.add(self.buffer.len() as DataSize);
            self.buffer.put_u16(key_len);
            self.buffer.extend_from_slice(key);
            self.buffer.put_u16(value_len);
            self.buffer.extend_from_slice(value);
            self.key_range.add(key);
            return true;
        }
        false
    }

    pub fn len(&self) -> usize {
        // Key range is not included to block, it is stored separately 
        self.buffer.len() + self.offsets.len() // 2 bytes for key length + 2 bytes for value length
    }

    pub fn write_to(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.buffer);
        self.offsets.write_to(buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_keys_and_finalize() {
        let mut block = Block::new(64);
        
        // +10 bytes for key and value
        // +4 bytes for key-val lengths
        // +2 bytes for offsets (with 2 bytes long footer)
        assert!(block.add(b"key1", b"value1"));
        assert_eq!(block.len(), 18);

        // +10 bytes for key and value
        // +4 bytes for key-val lengths
        // +2 bytes for offsets
        assert!(block.add(b"key2", b"value2"));
        assert_eq!(block.len(), 34);

        // +12 bytes for key and value
        // +4 bytes for key-val lengths
        // +2 bytes for offsets
        assert!(block.add(b"key10", b"value20"));
        assert_eq!(block.len(), 52);

        let mut buffer = Vec::new();
        block.write_to(&mut buffer);
        assert_eq!(buffer.len(), 52);
        assert_eq!(buffer, 
                   vec![
                       0, 4, b'k', b'e', b'y', b'1', // key1
                       0, 6, b'v', b'a', b'l', b'u', b'e', b'1', // value1
                       0, 4, b'k', b'e', b'y', b'2', // key2
                       0, 6, b'v', b'a', b'l', b'u', b'e', b'2', // value2
                       0, 5, b'k', b'e', b'y', b'1', b'0', // key10
                       0, 7, b'v', b'a', b'l', b'u', b'e', b'2', b'0',// value20
                       0, 0, // offset for key1
                       0, 14, // offset for key2
                       0, 28, // offset for key10
                       // Footer with offsets count (3)
                       0, 3,
                   ]);
    }

    #[test]
    fn test_max_size_constraint() {
        let mut block = Block::new(64);
        
        // +10 bytes for key and value
        // +4 bytes for key-val lengths
        // +2 bytes for offsets (with 2 bytes long footer)
        assert!(block.add(b"key1", b"value1"));
        assert_eq!(block.len(), 18);

        // +10 bytes for key and value
        // +4 bytes for key-val lengths
        // +2 bytes for offsets
        assert!(block.add(b"key2", b"value2"));
        assert_eq!(block.len(), 34);

        // +12 bytes for key and value
        // +4 bytes for key-val lengths
        // +2 bytes for offsets
        assert!(block.add(b"key10", b"value20"));
        assert_eq!(block.len(), 52);

        // +12 bytes for key and value
        // +4 bytes for key-val lengths
        // +2 bytes for offsets
        assert!(block.add(b"key", b"val"));
        assert_eq!(block.len(), 64);

        // Adding another key should fail due to size constraint
        assert!(!block.add(b"key4", b"val"));

        let mut buffer = Vec::new();
        block.write_to(&mut buffer);
        assert_eq!(buffer.len(), 64);
    }

    #[test]
    fn test_write_nothing_and_finalize() {
        let block = Block::new(64);
        
        // No data added, should be just 2 bytes long footer (number of items)
        assert_eq!(block.len(), 2);

        let mut buffer = Vec::new();
        block.write_to(&mut buffer);
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer, vec![0, 0]); // Footer with offsets count (0)
    }
}