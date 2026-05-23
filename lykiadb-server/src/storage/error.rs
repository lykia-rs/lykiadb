use lykiadb_common::error::InputError;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum StorageError {
    #[error("Only objects can be inserted to the collections")]
    InvalidValue,
}

impl From<StorageError> for InputError {
    fn from(value: StorageError) -> Self {
        let hint = match &value {
            StorageError::InvalidValue => "Ensure the value is a valid object",
        };

        InputError::new(&value.to_string(), hint, None)
    }
}
