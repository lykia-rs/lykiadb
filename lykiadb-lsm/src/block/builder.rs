use crate::{
    block::Block,
    meta::{MetaEntryOffset, MetaKeyRange},
};
use bytes::BufMut;

pub(crate) type DataKeyLen = u16;
pub(crate) type DataValueLen = u32;
pub(crate) type DataOffsetLen = u32;
pub(crate) const SIZEOF_DATA_KEY_LEN: usize = std::mem::size_of::<DataKeyLen>();
pub(crate) const SIZEOF_DATA_VALUE_LEN: usize = std::mem::size_of::<DataValueLen>();
pub(crate) const SIZEOF_DATA_OFFSET_LEN: usize = std::mem::size_of::<DataOffsetLen>();

pub(crate) struct BlockBuilder {
    max_size: usize,
    buffer: Vec<u8>,
    offsets: MetaEntryOffset,
    pub key_range: MetaKeyRange,
}

impl BlockBuilder {
    pub fn new(max_size: usize) -> Self {
        BlockBuilder {
            max_size,
            buffer: Vec::new(),
            offsets: MetaEntryOffset::new(None),
            key_range: MetaKeyRange::new(),
        }
    }

    pub fn add(&mut self, key: &[u8], value: &[u8]) -> bool {
        let key_len = key.len() as DataKeyLen;
        let value_len = value.len() as DataValueLen;
        let required_for_data =
            key_len as usize + value_len as usize + SIZEOF_DATA_KEY_LEN + SIZEOF_DATA_VALUE_LEN;
        let required_for_meta = SIZEOF_DATA_OFFSET_LEN; // Size of new offset
        if required_for_data + required_for_meta + self.len() <= self.max_size {
            self.offsets.add(self.buffer.len() as DataOffsetLen);
            self.buffer.put_u16(key_len);
            self.buffer.extend_from_slice(key);
            self.buffer.put_u32(value_len);
            self.buffer.extend_from_slice(value);
            self.key_range.add(key);
            return true;
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn len(&self) -> usize {
        // Key range is not included to block, it is stored separately
        self.buffer.len() + self.offsets.len()
    }

    pub fn write_to(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.buffer);
        self.offsets.write_to(buffer);
    }

    pub fn to_block(&self) -> Block {
        Block {
            buffer: self.buffer.clone(),
            offsets: self.offsets.to_vec(),
        }
    }
}

#[macro_export]
macro_rules! build_block {
    ( $( ($key:expr, $value:expr) ),* ) => {{
    let mut builder = crate::block::builder::BlockBuilder::new(4096);
    $( builder.add($key, $value); )*
        builder.to_block()
    }};
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_keys_and_finalize() {
        let mut block = BlockBuilder::new(64);

        // +8 bytes for key and value
        // +6 bytes for key-val lengths
        // +4 bytes for offsets (with 4 bytes long footer)
        assert!(block.add(b"key", b"value"));
        assert_eq!(block.len(), 22);

        // +10 bytes for key and value
        // +6 bytes for key-val lengths
        // +4 bytes for offsets
        assert!(block.add(b"key2", b"value2"));
        assert_eq!(block.len(), 42);

        // +12 bytes for key and value
        // +6 bytes for key-val lengths
        // +4 bytes for offsets
        assert!(block.add(b"key10", b"value20"));
        assert_eq!(block.len(), 64);

        let mut buffer = Vec::new();
        block.write_to(&mut buffer);
        assert_eq!(buffer.len(), 64);
        assert_eq!(
            buffer,
            vec![
                0, 3, b'k', b'e', b'y', // key
                0, 0, 0, 5, b'v', b'a', b'l', b'u', b'e', // value
                0, 4, b'k', b'e', b'y', b'2', // key2
                0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'2', // value2
                0, 5, b'k', b'e', b'y', b'1', b'0', // key10
                0, 0, 0, 7, b'v', b'a', b'l', b'u', b'e', b'2', b'0', // value20
                0, 0, 0, 0, // offset for key1
                0, 0, 0, 14, // offset for key2
                0, 0, 0, 30, // offset for key10
                // Footer with offsets count (3)
                0, 0, 0, 3,
            ]
        );
    }

    #[test]
    fn test_max_size_constraint() {
        let mut block = BlockBuilder::new(64);

        // +8 bytes for key and value
        // +6 bytes for key-val lengths
        // +4 bytes for offset (+4 bytes long footer)
        assert!(block.add(b"key", b"value"));
        assert_eq!(block.len(), 22);

        // +10 bytes for key and value
        // +6 bytes for key-val lengths
        // +4 bytes for offset
        assert!(block.add(b"key2", b"value2"));
        assert_eq!(block.len(), 42);

        // +12 bytes for key and value
        // +6 bytes for key-val lengths
        // +4 bytes for offset
        assert!(block.add(b"key10", b"value20"));
        assert_eq!(block.len(), 64);

        // Adding another key should fail due to size constraint
        assert!(!block.add(b"key4", b"val"));

        let mut buffer = Vec::new();
        block.write_to(&mut buffer);
        assert_eq!(buffer.len(), 64);
    }

    #[test]
    fn test_write_nothing_and_finalize() {
        let block = BlockBuilder::new(64);

        // No data added, should be just 4 bytes long footer (number of items)
        assert_eq!(block.len(), 4);

        let mut buffer = Vec::new();
        block.write_to(&mut buffer);
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer, vec![0, 0, 0, 0]); // Footer with offsets count (0)
    }

    #[test]
    fn test_as_block() {
        let mut builder = BlockBuilder::new(64);

        assert!(builder.add(b"key", b"value"));

        assert!(builder.add(b"key2", b"value2"));

        assert!(builder.add(b"key10", b"value20"));

        let block = builder.to_block();

        assert_eq!(
            block.buffer,
            vec![
                0, 3, b'k', b'e', b'y', // key
                0, 0, 0, 5, b'v', b'a', b'l', b'u', b'e', // value
                0, 4, b'k', b'e', b'y', b'2', // key2
                0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'2', // value2
                0, 5, b'k', b'e', b'y', b'1', b'0', // key10
                0, 0, 0, 7, b'v', b'a', b'l', b'u', b'e', b'2', b'0', // value20
            ]
        );

        assert_eq!(block.offsets, vec![0, 14, 30])
    }
}
