pub mod memory;

type ScanIterator<'a> = Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>;

pub trait Store<'a> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: Vec<u8>);
    fn delete(&mut self, key: &[u8]);
    fn scan(&'a self) -> ScanIterator<'a>;
    fn scan_prefix(&'a self, prefix: &'a [u8]) -> ScanIterator<'a>;
}
