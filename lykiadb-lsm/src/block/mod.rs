use crate::{block::builder::{DataOffsetLen, SIZEOF_DATA_OFFSET_LEN}, meta::MetaEntryOffset};
pub(crate) mod builder;
pub(crate) mod iterator;
use bytes::Buf;
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

    pub fn fetch_key_of(&self, idx: usize) -> &[u8] {
        let mid_offset = self.offsets[idx] as usize;
        let mut buf = &self.buffer[mid_offset..];
        let key_len = buf.get_u16() as usize;
        &buf[..key_len]
    }

    /// Binary search the key
    pub fn find_key_idx(&self, key: &[u8]) -> usize {
        let mut lo = 0;
        let mut hi = self.offsets.len();
        let mut cursor = lo;
        while lo < hi {
            let mid = (hi + lo)/2;
            let mid_key = self.fetch_key_of(mid);
            if key < mid_key {
                hi = mid
            }
            else if key > mid_key {
                lo = mid + 1;
                cursor = lo;
            }
            else {
                cursor = mid;
                break;
            }
        }

        cursor
    }

    pub fn from_buffer(buffer: &[u8]) -> Self {
        if buffer.len() < SIZEOF_DATA_OFFSET_LEN {
            panic!("Buffer is too short to read block summary");
        }

        let number_of_entries =
            DataOffsetLen::from_be_bytes(*buffer.last_chunk::<SIZEOF_DATA_OFFSET_LEN>().unwrap());
        let data_ends_at = buffer.len()
            - number_of_entries as usize * SIZEOF_DATA_OFFSET_LEN
            - SIZEOF_DATA_OFFSET_LEN;

        let offsets = MetaEntryOffset::from_buffer(
            &buffer[data_ends_at..buffer.len()-SIZEOF_DATA_OFFSET_LEN],
        );

        Block {
            buffer: buffer[..data_ends_at].to_vec(),
            offsets,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::build_block;
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

        let block = Block::from_buffer(&buffer);
        assert_eq!(block.buffer.len(), 48);
        assert_eq!(block.offsets.len(), 3);
        assert_eq!(block.offsets[0], 0);
        assert_eq!(block.offsets[1], 14);
        assert_eq!(block.offsets[2], 30);
    }

    #[test]
    fn test_find_key_idx() {
        let block = build_block![
            (b"1", b"value1"),
            (b"11", b"value11"),
            (b"13", b"value13"),
            (b"15", b"value15"),
            (b"17", b"value17"),
            (b"3", b"value3"),
            (b"5", b"value5"),
            (b"7", b"value7"),
            (b"9", b"value9")
        ];

        assert_eq!(block.find_key_idx(b"3"), 5);
        assert_eq!(block.find_key_idx(b"16"), 4);
        assert_eq!(block.find_key_idx(b"11"), 1);
        assert_eq!(block.find_key_idx(b"7"), 7);
    }
}
