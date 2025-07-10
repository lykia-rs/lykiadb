pub mod block;
pub mod memtable;
pub mod meta;
pub mod sstable;

type DataOffset = u16;
type DataSize = u16;
type Key = Vec<u8>;

pub(crate) const SIZEOF_U16: usize = std::mem::size_of::<u16>();
