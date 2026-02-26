use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct DocumentRef<'a> {
    bytes: &'a [u8],
}
impl<'a> DocumentRef<'a> {
    pub(crate) fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the document by its pointer address
        (self.bytes.as_ptr() as usize).hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct DocumentArrayRef<'a> {
    bytes: &'a [u8],
}
impl<'a> DocumentArrayRef<'a> {
    pub(crate) fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the document array by its pointer address
        (self.bytes.as_ptr() as usize).hash(state);
    }
}