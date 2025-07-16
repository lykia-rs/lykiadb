#[derive(Clone, PartialEq, Debug)]
pub struct Key<T: AsRef<[u8]>>(pub T);

impl Key<Vec<u8>> {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}