mod builder;

use std::{path::PathBuf};

use crate::{block::Block, meta::{MetaEntryOffset, MetaKeyRange}};

struct SSTable {
    file_path: PathBuf,
    data: Vec<Block>,
    key_range: MetaKeyRange,
    block_offsets: MetaEntryOffset,
}