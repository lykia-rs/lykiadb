use bytes::BufMut;
use crate::{meta::MetaKeyRange};

pub struct MetaBlockSummary {
    pub offset: u32,
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
