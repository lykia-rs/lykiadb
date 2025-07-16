use std::sync::Arc;

use bytes::Buf;

use crate::{block::{builder::{SIZEOF_DATA_KEY_LEN, SIZEOF_DATA_VALUE_LEN}, Block}, key::Key};

pub(crate) struct BlockIterator {
    block: Arc<Block>,
    idx: usize,
    key: Key<Vec<u8>>,
    value_span: (usize, usize),
}

impl IntoIterator for Block {
    type Item = Key<Vec<u8>>;
    type IntoIter = BlockIterator;

    fn into_iter(self) -> Self::IntoIter {
        BlockIterator::new(Arc::new(self))
    }
}

impl BlockIterator {
    fn new(block: Arc<Block>) -> BlockIterator {
        BlockIterator {
            block,
            idx: 0,
            key: Key(vec![]),
            value_span: (0, 0),
        }
    }

    fn _next(&mut self) {
        self.seek_idx(self.idx);
    }

    fn is_valid(&self) -> bool {
        !self.key.is_empty()
    }

    fn seek_idx(&mut self, idx: usize) {
        if self.block.offsets.len() <= idx {
            self.key.clear();
            self.idx = 0;
            self.value_span = (0, 0);
            return;
        }

        self.seek_offset(self.block.offsets[idx] as usize);
        self.idx = idx + 1;
    }
    
    fn seek_offset(&mut self, offset: usize) {
        let mut buf = &self.block.buffer[offset as usize..];
        let key_len = buf.get_u16() as usize;
        self.key = Key(buf[..key_len].to_vec());
        buf.advance(key_len);
        let value_len = buf.get_u32() as usize;
        let val_begin = offset + key_len + SIZEOF_DATA_KEY_LEN + SIZEOF_DATA_VALUE_LEN;
        let val_end = val_begin + value_len;
        self.value_span = (val_begin, val_end);
    }

    pub fn value(&self) -> Vec<u8> {
        self.block.buffer[self.value_span.0..self.value_span.1].to_vec()
    }

    /// Binary search the key
    pub fn seek_key(&mut self, key: &[u8]) {
        let cursor = self.block.find_key_idx(key);
        self.seek_idx(cursor);
    }
}

impl std::iter::Iterator for BlockIterator {
    type Item = Key<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        self._next();

        if self.is_valid() {
            return Some(self.key.clone())
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{build_block, key::Key};

    #[test]
    fn test_into_iter() {
        let block = build_block![
            (b"key1", b"value1"),
            (b"key20", b"value30"),
            (b"key300", b"value500")
        ];

        let mut output = Vec::new();
        for item in block {
            output.push(item.0);
        }

        assert_eq!(output, vec![b"key1".to_vec(), b"key20".to_vec(), b"key300".to_vec()]);
    }

    #[test]
    fn test_getting_value() {
        let block = build_block![
            (b"key1", b"value1"),
            (b"key20", b"value30"),
            (b"key300", b"value500")
        ];

        let mut iter = block.into_iter();
        assert_eq!(iter.next().unwrap(), Key(b"key1".to_vec()));
        assert_eq!(iter.value(), b"value1".to_vec());
        assert_eq!(iter.next().unwrap(), Key(b"key20".to_vec()));
        assert_eq!(iter.value(), b"value30".to_vec());
        assert_eq!(iter.next().unwrap(), Key(b"key300".to_vec()));
        assert_eq!(iter.value(), b"value500".to_vec());
    }

    #[test]
    fn test_seek_key() {
        let block = build_block![
            (b"key1", b"value1"),
            (b"key20", b"value30"),
            (b"key300", b"value300"),
            (b"key4000", b"value4000"),
            (b"key5000", b"value5000"),
            (b"key600", b"value600")
        ];

        let mut iter = block.into_iter();
        iter.seek_key(b"key5000");
        assert_eq!(iter.value(), b"value5000".to_vec());
    }
    
    #[test]
    fn test_seek_closest_key() {
        let block = build_block![
            (b"1", b"value1"),
            (b"3", b"value3"),
            (b"5", b"value5")
        ];

        let mut iter = block.into_iter();

        iter.seek_key(b"3");
        assert_eq!(iter.value(), b"value3".to_vec());

        iter.seek_key(b"2");
        assert_eq!(iter.value(), b"value3".to_vec());

        iter.seek_key(b"4");
        assert_eq!(iter.value(), b"value5".to_vec());

        iter.seek_key(b"1");
        assert_eq!(iter.value(), b"value1".to_vec());

        iter.seek_key(b"5");
        assert_eq!(iter.value(), b"value5".to_vec());
    }

    #[test]
    fn test_seek_closest_key_2() {
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

        let mut iter = block.into_iter();
        iter.seek_key(b"2");
        assert_eq!(iter.value(), b"value3".to_vec());

        iter.seek_key(b"16");
        assert_eq!(iter.value(), b"value17".to_vec());

        iter.seek_key(b"11");
        assert_eq!(iter.value(), b"value11".to_vec());

        iter.seek_key(b"7");
        assert_eq!(iter.value(), b"value7".to_vec());
    }
}