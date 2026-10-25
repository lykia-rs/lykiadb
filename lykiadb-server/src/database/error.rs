use lykiadb_common::error::InputError;
use serde::{Deserialize, Serialize};

use crate::storage::error::StorageEngineError;

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseError {
    #[error("Only objects can be inserted to the collections")]
    InvalidValue,
    #[error("Storage engine error: {0}")]
    Engine(StorageEngineError),
}

impl From<DatabaseError> for InputError {
    fn from(value: DatabaseError) -> Self {
        let hint = match &value {
            DatabaseError::InvalidValue => "Ensure the value is a valid object",
            DatabaseError::Engine(_) => "An error occurred in the storage engine",
        };

        InputError::new(&value.to_string(), hint, None)
    }
}
