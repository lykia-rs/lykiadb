pub mod error;
pub mod memory;

pub type IteratorItem = Result<(Vec<u8>, Vec<u8>), error::StorageEngineError>;

pub trait StorageEngineScanIterator<'a>: Iterator<Item = IteratorItem> + 'a {}

impl<'a, I: Iterator<Item = IteratorItem> + 'a> StorageEngineScanIterator<'a> for I {}

pub trait StorageEngine<'a> {
    type ScanIterator: StorageEngineScanIterator<'a>;

    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: Vec<u8>);
    fn delete(&mut self, key: &[u8]);
    fn scan(&'a self) -> Self::ScanIterator;
}
