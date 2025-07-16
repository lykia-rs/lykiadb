pub struct Key<T: AsRef<[u8]>>(pub T);

impl Key<Vec<u8>> {
    pub fn clear(&mut self) {
        self.0.clear();
    }
}