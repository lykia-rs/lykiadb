mod builder;
use std::{fs::File, os::unix::fs::FileExt, path::PathBuf};

use bytes::Buf;

use crate::{
    block::builder::SIZEOF_DATA_OFFSET_LEN,
    meta::{MetaBlockSummary, MetaKeyRange},
};
#[derive(PartialEq, Debug)]
struct SSTable {
    handle: FileHandle,
    key_range: MetaKeyRange,
    block_summaries: Vec<MetaBlockSummary>,
}

impl SSTable {

    /// Binary search the key
    pub fn find_block_idx(&self, key: &[u8]) -> usize {
        self.block_summaries
            .partition_point(|meta| meta.key_range.min_key.as_slice() <= key)
            .saturating_sub(1)
    }
    
    pub fn open(file_path: &PathBuf) -> Result<SSTable, std::io::Error> {
        let handle = FileHandle::open(file_path)?;

        let mut size_buffer = &handle.read_from_end(SIZEOF_DATA_OFFSET_LEN)?[..];

        if handle.size < SIZEOF_DATA_OFFSET_LEN as u64 {
            panic!("Buffer too short to read table summary");
        }

        let data_ends_at = size_buffer.get_u32() as usize;

        let mut meta_buffer = &handle.read(
            data_ends_at,
            handle.size as usize - data_ends_at - SIZEOF_DATA_OFFSET_LEN,
        )?[..];

        let number_of_blocks = meta_buffer.get_u32() as usize;

        let block_summaries =
            MetaBlockSummary::from_buffer(meta_buffer, number_of_blocks);

        Ok(SSTable {
            handle,
            key_range: block_summaries
                .first()
                .unwrap()
                .key_range
                .merge(&block_summaries.last().unwrap().key_range),
            block_summaries,
        })
    }
}

#[derive(Debug)]
struct FileHandle {
    path: PathBuf,
    inner_handle: File,
    size: u64,
}

impl FileHandle {
    fn open(file_path: &PathBuf) -> Result<FileHandle, std::io::Error> {
        let file = File::options().read(true).write(false).open(file_path)?;
        let size = file.metadata()?.len();

        Ok(FileHandle {
            path: file_path.clone(),
            inner_handle: file,
            size,
        })
    }

    fn read(&self, start: usize, len: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = vec![0; len];
        self.inner_handle.read_exact_at(&mut buf, start as u64)?;

        Ok(buf)
    }

    fn read_from_end(&self, bytes: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = vec![0; bytes];
        self.inner_handle
            .read_exact_at(&mut buf, self.size - bytes as u64)?;

        Ok(buf)
    }
}

impl PartialEq for FileHandle {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::sstable::{SSTable, builder::SSTableBuilder};

    #[test]
    fn test_open() {
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

        let read_from_file = SSTable::open(&file_path).unwrap();

        assert_eq!(initial, read_from_file);
    }


    #[test]
    fn test_sstable() {
        let mut builder = SSTableBuilder::new(PathBuf::from("/tmp/test_sstable_with_multiple_blocks"), 64);

        // 0
        builder.add(b"key1", b"value1");
        builder.add(b"key10", b"value10");

        // 1
        builder.add(b"key11", b"value11");
        builder.add(b"key12", b"value12");

        // 2
        builder.add(b"key2", b"value2");
        builder.add(b"key3", b"value3");
        builder.add(b"key4", b"value4");

        // 3
        builder.add(b"key5", b"value5");
        builder.add(b"key6", b"value6");
        builder.add(b"key7", b"value7");

        // 4
        builder.add(b"key8", b"value8");
        builder.add(b"key9", b"value9");

        let table = builder.build().unwrap();

        assert_eq!(table.find_block_idx(b"key1"), 0);
        assert_eq!(table.find_block_idx(b"key10"), 0);
        
        assert_eq!(table.find_block_idx(b"key11"), 1);
        assert_eq!(table.find_block_idx(b"key12"), 1);
        
        assert_eq!(table.find_block_idx(b"key2"), 2);
        assert_eq!(table.find_block_idx(b"key3"), 2);
        assert_eq!(table.find_block_idx(b"key4"), 2);
        
        assert_eq!(table.find_block_idx(b"key5"), 3);
        assert_eq!(table.find_block_idx(b"key6"), 3);
        assert_eq!(table.find_block_idx(b"key7"), 3);

        assert_eq!(table.find_block_idx(b"key8"), 4);
        assert_eq!(table.find_block_idx(b"key9"), 4);
    }
}
