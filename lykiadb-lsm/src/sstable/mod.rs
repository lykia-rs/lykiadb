mod builder;
use std::fmt::Debug;
use std::{fs::File, os::unix::fs::FileExt, path::PathBuf, sync::Arc};

use bytes::Buf;
use moka::sync::Cache;

use crate::{
    block::{Block, builder::SIZEOF_DATA_OFFSET_LEN},
    meta::{MetaBlockSummary, MetaKeyRange},
};

struct SSTable {
    file_handle: FileHandle,
    data_ends_at: usize,
    key_range: MetaKeyRange,
    block_summaries: Vec<MetaBlockSummary>,
    block_cache: Option<Cache<usize, Arc<Block>>>,
}

impl PartialEq for SSTable {
    fn eq(&self, other: &Self) -> bool {
        self.file_handle == other.file_handle
            && self.data_ends_at == other.data_ends_at
            && self.key_range == other.key_range
            && self.block_summaries == other.block_summaries
    }
}

impl Debug for SSTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SSTable")
            .field("file_handle", &self.file_handle)
            .field("data_ends_at", &self.data_ends_at)
            .field("key_range", &self.key_range)
            .field("block_summaries", &self.block_summaries)
            .finish()
    }
}

impl SSTable {
    /// Binary search the key
    pub fn find_block_idx(&self, key: &[u8]) -> usize {
        self.block_summaries
            .partition_point(|meta| meta.key_range.min_key.as_slice() <= key)
            .saturating_sub(1)
    }

    pub fn read_block(&self, idx: usize) -> Result<Arc<Block>, std::io::Error> {
        let start_offset = self.block_summaries[idx].offset as usize;
        let len = self
            .block_summaries
            .get(idx + 1)
            .map_or_else(|| self.data_ends_at, |x| x.offset as usize)
            - start_offset;

        let buffer = self.file_handle.read(start_offset, len)?;
        Ok(Arc::from(Block::from_buffer(&buffer)))
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

        let block_summaries = MetaBlockSummary::from_buffer(meta_buffer, number_of_blocks);

        Ok(SSTable {
            file_handle: handle,
            data_ends_at,
            key_range: block_summaries
                .first()
                .unwrap()
                .key_range
                .merge(&block_summaries.last().unwrap().key_range),
            block_summaries,
            block_cache: None,
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

    use crate::{
        block::iterator::BlockIterator,
        sstable::{SSTable, builder::SSTableBuilder},
    };

    #[test]
    fn test_open() {
        let file_path = PathBuf::from("/tmp/test_sstable_open");
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
    fn test_sstable_find_idx() {
        let mut builder = SSTableBuilder::new(PathBuf::from("/tmp/test_sstable_find_idx"), 64);

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

    #[test]
    fn test_sstable_read_block() {
        let mut builder = SSTableBuilder::new(PathBuf::from("/tmp/test_sstable_read_block"), 64);

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

        let mut first_block_iter = BlockIterator::new(table.read_block(0).unwrap());
        first_block_iter.seek_key(b"key1");
        assert_eq!(first_block_iter.value(), b"value1");
        first_block_iter.seek_key(b"key10");
        assert_eq!(first_block_iter.value(), b"value10");

        let block = table.read_block(2).unwrap();
        assert_eq!(block.fetch_key_of(0), b"key2");
        assert_eq!(block.fetch_key_of(1), b"key3");
        assert_eq!(block.fetch_key_of(2), b"key4");

        let mut last_block_iter = BlockIterator::new(table.read_block(4).unwrap());
        last_block_iter.seek_key(b"key8");
        assert_eq!(last_block_iter.value(), b"value8");
        last_block_iter.seek_key(b"key9");
        assert_eq!(last_block_iter.value(), b"value9");
    }
}
