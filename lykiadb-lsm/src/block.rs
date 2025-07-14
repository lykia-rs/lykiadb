use crate::{block::builder::{DataOffsetLen, SIZEOF_DATA_OFFSET_LEN}};
pub(crate) mod builder;

pub(crate) struct Block {
    buffer: Vec<u8>,
    offsets: Vec<DataOffsetLen>,
}

impl Block {
    pub fn new() -> Self {
        Block {
            buffer: Vec::new(),
            offsets: Vec::new(),
        }
    }

    pub fn from_buffer(buffer: &[u8]) -> Self {
        if buffer.len() < SIZEOF_DATA_OFFSET_LEN {
            panic!("Buffer too short to read block summary");
        }

        let number_of_entries = DataOffsetLen::from_be_bytes(*buffer.last_chunk::<SIZEOF_DATA_OFFSET_LEN>().unwrap());
        let data_ends_at = buffer.len() - number_of_entries as usize * SIZEOF_DATA_OFFSET_LEN - SIZEOF_DATA_OFFSET_LEN;

        let mut offsets = Vec::with_capacity(number_of_entries as usize);
        
        for i in 0..number_of_entries {
            let offset = DataOffsetLen::from_be_bytes(*buffer[
                (data_ends_at + i as usize * SIZEOF_DATA_OFFSET_LEN)..
                (data_ends_at + (i + 1) as usize * SIZEOF_DATA_OFFSET_LEN)
                ].last_chunk::<SIZEOF_DATA_OFFSET_LEN>().unwrap());

            offsets.push(offset);
        }

        Block {
            buffer: buffer[..data_ends_at].to_vec(),
            offsets,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from_buffer() {
        let buffer = vec![
            0, 3, b'k', b'e', b'y', // key
            0, 0, 0, 5, b'v', b'a', b'l', b'u', b'e', // value
            0, 4, b'k', b'e', b'y', b'2', // key2
            0, 0, 0, 6, b'v', b'a', b'l', b'u', b'e', b'2', // value2
            0, 5, b'k', b'e', b'y', b'1', b'0', // key10
            0, 0, 0, 7, b'v', b'a', b'l', b'u', b'e', b'2', b'0', // value20
            0, 0, 0, 0, // offset for key1
            0, 0, 0, 14, // offset for key2
            0, 0, 0, 30, // offset for key10
            // Footer with offsets count (3)
            0, 0, 0, 3,
        ];

        let block   = Block::from_buffer(&buffer);
        assert_eq!(block.buffer.len(), 48);
        assert_eq!(block.offsets.len(), 3);
        assert_eq!(block.offsets[0], 0);
        assert_eq!(block.offsets[1], 14);
        assert_eq!(block.offsets[2], 30);
    }
}