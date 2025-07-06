use crate::{meta::MetaKeyRange, DataOffset};

pub struct MetaBlockSummary {
    pub offset: DataOffset,
    pub key_range: MetaKeyRange,
}