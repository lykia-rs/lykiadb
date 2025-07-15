
use std::path::PathBuf;

use crate::{block::builder::{DataOffsetLen, SIZEOF_DATA_OFFSET_LEN}, meta::MetaBlockSummary, sstable::SSTable};

pub(crate) struct SSTableReader;

impl SSTableReader {
    pub fn read(file_path: &PathBuf) -> Result<SSTable, std::io::Error>  {
        let buffer = std::fs::read(&file_path).unwrap();

        if buffer.len() < SIZEOF_DATA_OFFSET_LEN {
            panic!("Buffer too short to read table summary");
        }

        let data_ends_at =
            DataOffsetLen::from_be_bytes(*buffer.last_chunk::<SIZEOF_DATA_OFFSET_LEN>().unwrap()) as usize;

        let number_of_blocks = DataOffsetLen::from_be_bytes(*buffer[data_ends_at..].first_chunk::<SIZEOF_DATA_OFFSET_LEN>().unwrap());

        let block_summaries = MetaBlockSummary::from_buffer(&buffer[data_ends_at + 4..], number_of_blocks as usize);

        Ok(SSTable {
            file_path: file_path.clone(),
            key_range: block_summaries
                .first()
                .unwrap()
                .key_range
                .merge(&block_summaries.last().unwrap().key_range),
            block_summaries,
        })
    }
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::sstable::{builder::SSTableBuilder, reader::SSTableReader};

    #[test]
    fn test_open(){
        let file_path = PathBuf::from("/tmp/test_sstable_with_multiple_blocks");
        let mut builder = SSTableBuilder::new(file_path.clone(), 64);

        // Block 1: 3 key-value pairs (fills the 64-byte block)
        builder.add(b"key1", b"value1");
        builder.add(b"key2", b"value2");
        builder.add(b"key3", b"value3");

        // Block 2: 3 more key-value pairs (creates second block)
        builder.add(b"key4", b"value4");
        builder.add(b"key5", b"value5");
        builder.add(b"key6", b"value6");

        // Block 3: 3 more key-value pairs (creates third block)
        builder.add(b"key7", b"value7");
        builder.add(b"key8", b"value8");
        builder.add(b"key9", b"value9");

        let initial = builder.build().unwrap();

        let read_from_file = SSTableReader::read(&file_path).unwrap();

        assert_eq!(initial, read_from_file);
    }
}