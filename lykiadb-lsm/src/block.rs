use crate::meta::MetaEntryOffset;

pub mod builder;
pub struct Block {
    buffer: Vec<u8>,
    offsets: MetaEntryOffset,
}

impl Block {
    pub fn new() -> Self {
        Block {
            buffer: Vec::new(),
            offsets: MetaEntryOffset::new(None),
        }
    }
}
