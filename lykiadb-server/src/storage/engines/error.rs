use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum StorageEngineError {
    #[error("Unknown error")]
    UnknownError
}
