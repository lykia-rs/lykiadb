pub mod block;
pub mod memtable;
pub mod meta;
pub mod sstable;

type DataSize = u16;
type Key = Vec<u8>;