use crate::{block::builder::DataOffsetLen, meta::MetaKeyRange};
use bytes::{Buf, BufMut};

#[derive(Clone, Debug, PartialEq)]
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

    pub fn from_buffer(mut buffer: &[u8], number_of_blocks: usize) -> Vec<Self> {
        let mut summaries = Vec::with_capacity(number_of_blocks);

        for _ in 0..number_of_blocks {
            let start_offset = buffer.get_u32();
            let min_key_len = buffer.get_u16();
            let min_key = buffer.copy_to_bytes(min_key_len as usize).to_vec();
            let max_key_len = buffer.get_u16();
            let max_key = buffer.copy_to_bytes(max_key_len as usize).to_vec();

            summaries.push(MetaBlockSummary {
                offset: start_offset,
                key_range: MetaKeyRange::build(min_key, max_key),
            });
        }

        summaries
    }
}
