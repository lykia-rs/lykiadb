use lykiadb_common::error::InputError;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum EngineError {
    #[error("Only objects can be inserted to the collections")]
    InvalidValue,
}

impl From<EngineError> for InputError {
    fn from(value: EngineError) -> Self {
        let hint = match &value {
            EngineError::InvalidValue => "Ensure the value is a valid object",
        };

        InputError::new(&value.to_string(), hint, None)
    }
}
