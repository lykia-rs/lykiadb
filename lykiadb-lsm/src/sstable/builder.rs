use bytes::BufMut;
use std::path::PathBuf;

use crate::{block::Block, meta::MetaBlockSummary};

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
        self.current_block.write_to(&mut self.buffer);
        self.block_summaries.push(MetaBlockSummary {
            offset: self.buffer.len() as u32,
            key_range: self.current_block.key_range.clone(),
        });
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
        self.buffer.put_u32(self.block_summaries.len() as u32);
        for meta in &self.block_summaries {
            meta.write_to(&mut self.buffer);
        }
        self.buffer.put_u32(meta_offset as u32);
        std::fs::write(&self.file_path, &self.buffer)?;
        Ok(())
    }
}
