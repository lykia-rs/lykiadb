use crate::{database::storage::{Key, StorageIteratorItem}, execution::error::ExecutionError, value::RV};
use crate::storage::StorageEngine;

pub mod error;
pub mod storage;

pub struct Database<S: for<'a> StorageEngine<'a>> {
    storage: S,
}

pub trait Transaction<'a> {
    fn get(&self, key: Key<'_>) -> Option<RV<'_>>;
    fn set(&mut self, key: Key<'_>, value: RV<'_>) -> Result<(), ExecutionError>;
    fn delete(&mut self, key: Key<'_>);
    fn scan(&'a self, prefix: Key<'_>) -> impl Iterator<Item = StorageIteratorItem<'a>>;
}