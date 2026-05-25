use crate::{database::{Key, StorageIteratorItem}, execution::error::ExecutionError, value::RV};

pub trait Transaction<'a> {
    fn get(&self, key: Key<'_>) -> Option<RV<'_>>;
    fn set(&mut self, key: Key<'_>, value: RV<'_>) -> Result<(), ExecutionError>;
    fn delete(&mut self, key: Key<'_>);
    fn scan(&'a self, prefix: Key<'_>) -> impl Iterator<Item = StorageIteratorItem<'a>>;
}