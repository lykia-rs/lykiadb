use crate::{database::{error::DatabaseError, storage::{Key, StorageIteratorItem}}, value::RV};

pub mod error;
pub mod storage;
pub mod simple;

pub trait Database {
    type Transaction: Transaction;
    fn begin(&'_ self) -> Self::Transaction;
}

pub type TransactionActionResult<T> = Result<T, error::DatabaseError>;

pub trait Transaction {
    fn get(&self, key: Key<'_>) -> TransactionActionResult<Option<RV<'_>>>;
    fn set(&mut self, key: Key<'_>, value: RV<'_>) -> TransactionActionResult<()>;
    fn delete(&mut self, key: Key<'_>) -> TransactionActionResult<()>;
    fn scan<'a>(&self, prefix: Key<'a>) -> TransactionActionResult<impl Iterator<Item = StorageIteratorItem<'a>> + 'a>;
}

impl<T> From<std::sync::PoisonError<T>> for DatabaseError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        panic!("{err}")
    }
}
