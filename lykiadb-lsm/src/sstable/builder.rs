use bytes::BufMut;
use std::path::PathBuf;

use crate::{block::{Block, DataOffsetLen}, meta::MetaBlockSummary};

struct SSTableBuilder {
    file_path: PathBuf,
    max_block_size: usize,
    //
    buffer: Vec<u8>,
    block_summaries: Vec<MetaBlockSummary>,
    //
    current_block: Block,
}

impl SSTableBuilder {
    pub fn new(file_path: PathBuf, max_block_size: usize) -> Self {
        SSTableBuilder {
            file_path,
            max_block_size,
            buffer: Vec::new(),
            block_summaries: Vec::new(),
            current_block: Block::new(max_block_size),
        }
    }

    fn finalize_block(&mut self) {
        self.block_summaries.push(MetaBlockSummary {
            offset: self.buffer.len() as DataOffsetLen,
            key_range: self.current_block.key_range.clone(),
        });
        self.current_block.write_to(&mut self.buffer);
    }

    pub fn add(&mut self, key: &[u8], value: &[u8]) {
        let written = self.current_block.add(key, value);
        if !written {
            self.finalize_block();
            self.current_block = Block::new(self.max_block_size);
            self.current_block.add(key, value);
        }
    }

    pub fn write(&mut self) -> std::io::Result<()> {
        self.finalize_block();
        let meta_offset = self.buffer.len();
        self.buffer.put_u32(self.block_summaries.len() as DataOffsetLen);
        for meta in &self.block_summaries {
            meta.write_to(&mut self.buffer);
        }
        self.buffer.put_u32(meta_offset as DataOffsetLen);
        std::fs::write(&self.file_path, &self.buffer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sstable() {
        let file_path = PathBuf::from("/tmp/test_sstable");
        let mut builder = SSTableBuilder::new(file_path.clone(), 64);

        builder.add(b"key", b"value");

        builder.add(b"key2", b"value2");

        builder.add(b"key10", b"value20");

        builder.write().unwrap();

        let buffer = std::fs::read(&file_path).unwrap();
        assert_eq!(buffer, vec![
            // <block id=0>---------------------------
            0, 3, b'k', b'e', b'y', // key
            0, 0, 0, 5, b'v', b'a', b'l', b'u', b'e', // value
            0, 4, b'k', b'e', b'y', b'2', // key2
            0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'2', // value2
            0, 5, b'k', b'e', b'y', b'1', b'0', // key10
            0, 0, 0, 7, b'v', b'a', b'l', b'u', b'e', b'2', b'0', // value20
            0, 0, 0, 0, // offset for key1
            0, 0, 0, 14, // offset for key2
            0, 0, 0, 30, // offset for key10
            0, 0, 0, 3, // Footer with offsets count
            // </block>-------------------------------
            
            0, 0, 0, 1, // Number of blocks

            // <meta id=0>----------------------------
            0, 0, 0, 0, // Block start offset
            0, 3, b'k', b'e', b'y', // key (min key)
            0, 4, b'k', b'e', b'y', b'2', // key2 (max key)
            // </meta>--------------------------------

            0, 0, 0, 64, // Meta section offset
        ]);
    }

    #[test]
    fn test_sstable_with_mulitple_blocks() {
        let file_path = PathBuf::from("/tmp/test_sstable_with_mulitple_blocks");
        let mut builder = SSTableBuilder::new(file_path.clone(), 64);

        // Block 1: 3 key-value pairs (fills the 64-byte block)
        builder.add(b"key1", b"value1");
        builder.add(b"key2", b"value2");
        builder.add(b"key3", b"value3");

        // Block 2: 3 more key-value pairs (creates second block)
        builder.add(b"key4", b"value4");
        builder.add(b"key5", b"value5");
        builder.add(b"key6", b"value6");

        // Block 3: 3 more key-value pairs (creates third block)
        builder.add(b"key7", b"value7");
        builder.add(b"key8", b"value8");
        builder.add(b"key9", b"value9");

        builder.write().unwrap();

        let buffer = std::fs::read(&file_path).unwrap();
        assert_eq!(buffer, vec![
            // <block id=0>---------------------------
            0, 4, b'k', b'e', b'y', b'1', // key1
            0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'1', // value1
            0, 4, b'k', b'e', b'y', b'2', // key2
            0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'2', // value2
            0, 4, b'k', b'e', b'y', b'3', // key3
            0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'3', // value3
            0, 0, 0, 0, // offset for key1
            0, 0, 0, 16, // offset for key2
            0, 0, 0, 32, // offset for key10
            0, 0, 0, 3, // Footer with offsets count
            // </block>-------------------------------

            // <block id=1>---------------------------
            0, 4, b'k', b'e', b'y', b'4', // key4
            0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'4', // value4
            0, 4, b'k', b'e', b'y', b'5', // key5
            0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'5', // value5
            0, 4, b'k', b'e', b'y', b'6', // key6
            0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'6', // value6
            0, 0, 0, 0, // offset for key1
            0, 0, 0, 16, // offset for key2
            0, 0, 0, 32, // offset for key10
            0, 0, 0, 3, // Footer with offsets count
            // </block>-------------------------------

            // <block id=2>---------------------------
            0, 4, b'k', b'e', b'y', b'7', // key7
            0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'7', // value7
            0, 4, b'k', b'e', b'y', b'8', // key8
            0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'8', // value8
            0, 4, b'k', b'e', b'y', b'9', // key9
            0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'9', // value9
            0, 0, 0, 0, // offset for key1
            0, 0, 0, 16, // offset for key2
            0, 0, 0, 32, // offset for key10
            0, 0, 0, 3, // Footer with offsets count
            // </block>-------------------------------
            
            0, 0, 0, 3, // Number of blocks

            // <meta id=0>----------------------------
            0, 0, 0, 0, // Block start offset
            0, 4, b'k', b'e', b'y', b'1', // key1 (min key)
            0, 4, b'k', b'e', b'y', b'3', // key3 (max key)
            // </meta>--------------------------------

            // <meta id=1>----------------------------
            0, 0, 0, 64, // Block start offset
            0, 4, b'k', b'e', b'y', b'4', // key4 (min key)
            0, 4, b'k', b'e', b'y', b'6', // key6 (max key)
            // </meta>--------------------------------

            // <meta id=2>----------------------------
            0, 0, 0, 128, // Block start offset
            0, 4, b'k', b'e', b'y', b'7', // key7 (min key)
            0, 4, b'k', b'e', b'y', b'9', // key9 (max key)
            // </meta>--------------------------------

            0, 0, 0, 192, // Meta section offset
        ]);
    }
}