use std::{path::PathBuf};

use crate::{block::Block, meta::{MetaBlockSummary, MetaKeyRange}};

struct SSTableWriter {
    file_path: PathBuf,
    //
    buffer: Vec<u8>,
    key_range: MetaKeyRange,
    block_summaries: Vec<MetaBlockSummary>,
    //
    current_block: Block,
}

impl SSTableWriter {
    pub fn new(file_path: PathBuf) -> Self {
        SSTableWriter { 
            file_path, 
            buffer: Vec::new(), 
            key_range: MetaKeyRange::new(),
            block_summaries: Vec::new(),
            current_block: Block::new(4096),
        }
    }

    fn finalize_block(&mut self) {
        self.current_block.write_to(&mut self.buffer);
        self.block_summaries.push(MetaBlockSummary {
            offset: self.buffer.len() as u16,
            key_range: self.current_block.key_range.clone(),
        });
    }

    pub fn add(&mut self, key: &[u8], value: &[u8]) {
        let written = self.current_block.add(key, value);
        if !written {
            self.finalize_block();
            self.current_block = Block::new(4096);
            self.current_block.add(key, value);
        }
        self.key_range.add(key);
    }

    pub fn write(&mut self) -> std::io::Result<()> {  
        self.finalize_block();
        self.key_range.write_to(&mut self.buffer);
        std::fs::write(&self.file_path, &self.buffer)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_write_blocks_and_finalize_0() {
    }
}