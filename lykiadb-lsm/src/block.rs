use bytes::BufMut;
use crate::{meta::{MetaEntryOffset, MetaKeyRange}, DataSize};

const MAX_BLOCK_SIZE: usize = 4096;

pub struct Block {
    len: usize,
    buffer: Vec<u8>,
    offsets: MetaEntryOffset,
    pub key_range: MetaKeyRange,
}

impl Block {
    pub fn new() -> Self {
        Block {
            len: 0,
            buffer: Vec::new(),
            offsets: MetaEntryOffset::new(None),
            key_range: MetaKeyRange::new(),
        }
    }

    pub fn add(&mut self, key: &[u8], value: &[u8], buffer: &mut Vec<u8>) -> bool {
        let key_len = key.len() as DataSize;
        let value_len = value.len() as DataSize;
        let required_for_data = key_len as usize + value_len as usize + 4;
        let required_for_meta = 2; // Size of new offset
        if required_for_data + required_for_meta < MAX_BLOCK_SIZE - self.buffer.len() {
            buffer.put_u16(key_len);
            buffer.extend_from_slice(key);
            buffer.put_u16(value_len);
            buffer.extend_from_slice(value);
            self.key_range.add(key);
            self.offsets.add(buffer.len() as DataSize);
            self.len += 1;
            return true;
        }
        false
    }

    pub fn len(&self) -> usize {
        // Key range is not included to block, it is stored separately 
        self.buffer.len() + self.offsets.len() +  self.len * 4 // 2 bytes for key length + 2 bytes for value length
    }

    pub fn write_to(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.buffer);
        self.offsets.write_to(buffer);
    }
}