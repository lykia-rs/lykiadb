use crate::{block::builder::DataOffsetLen, meta::MetaKeyRange};
use bytes::BufMut;

#[derive(Clone)]
pub(crate) struct MetaBlockSummary {
    pub offset: DataOffsetLen,
    pub key_range: MetaKeyRange,
}

impl MetaBlockSummary {
    pub fn write_to(&self, buffer: &mut Vec<u8>) {
        buffer.put_u32(self.offset);
        self.key_range.write_to(buffer);
    }

    pub fn len(&self) -> usize {
        4 + self.key_range.len()
    }
}
