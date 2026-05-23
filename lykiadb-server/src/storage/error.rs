use lykiadb_common::error::InputError;
use serde::{Deserialize, Serialize};

use crate::storage::engines::error::StorageEngineError;

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum StorageError {
    #[error("Only objects can be inserted to the collections")]
    InvalidValue,
    #[error("Storage engine error: {0}")]
    Engine(StorageEngineError),
}

impl From<StorageError> for InputError {
    fn from(value: StorageError) -> Self {
        let hint = match &value {
            StorageError::InvalidValue => "Ensure the value is a valid object",
            StorageError::Engine(_) => "An error occurred in the storage engine",
        };

        InputError::new(&value.to_string(), hint, None)
    }
}
