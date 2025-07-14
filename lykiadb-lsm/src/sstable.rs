mod builder;

use std::path::PathBuf;

use crate::meta::{MetaBlockSummary, MetaKeyRange};

struct SSTable {
    file_path: PathBuf,
    key_range: MetaKeyRange,
    block_summaries: Vec<MetaBlockSummary>,
}
