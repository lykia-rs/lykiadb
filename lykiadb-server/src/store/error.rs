use lykiadb_common::error::InputError;
use lykiadb_lang::ast::Span;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum StoreError {
    #[error("Unspecified error at {span:?}")]
    UnspecifiedError { span: Span },
}

impl From<StoreError> for InputError {
    fn from(value: StoreError) -> Self {
        let (hint, sp) = match &value {
            StoreError::UnspecifiedError { span } => (
                "An unspecified error occurred in the store".to_string(),
                *span,
            ),
        };

        InputError::new(&value.to_string(), &hint, Some(sp.into()))
    }
}
