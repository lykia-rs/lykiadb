use lykiadb_common::error::InputError;
use lykiadb_lang::ast::Span;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum StorageEngineError {
    #[error("Unspecified error at {span:?}")]
    UnspecifiedError { span: Span },
}

impl From<StorageEngineError> for InputError {
    fn from(value: StorageEngineError) -> Self {
        let (hint, sp) = match &value {
            StorageEngineError::UnspecifiedError { span } => (
                "An unspecified error occurred in the store".to_string(),
                *span,
            ),
        };

        InputError::new(&value.to_string(), &hint, Some(sp.into()))
    }
}
