mod builder;
mod reader;

use std::path::PathBuf;

use crate::{meta::{MetaBlockSummary, MetaKeyRange}, sstable::reader::SSTableReader};
#[derive(PartialEq, Debug)]
struct SSTable {
    file_path: PathBuf,
    key_range: MetaKeyRange,
    block_summaries: Vec<MetaBlockSummary>,
}

impl SSTable {
    pub fn open(file_path: &PathBuf) -> Result<SSTable, std::io::Error> {
        SSTableReader::read(file_path)
    }
}
