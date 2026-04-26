pub mod memory;
pub mod error;
pub trait StoreScanIterator<'a>: Iterator<Item = Result<(Vec<u8>, Vec<u8>), error::StoreError>> + 'a {}

impl<'a, I: Iterator<Item = Result<(Vec<u8>, Vec<u8>), error::StoreError>> + 'a> StoreScanIterator<'a> for I {}

pub trait Store<'a> {
    type ScanIterator: StoreScanIterator<'a>;

    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: Vec<u8>);
    fn delete(&mut self, key: &[u8]);
    fn scan(&'a self) -> Self::ScanIterator;
}
