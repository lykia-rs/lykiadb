use lykiadb_common::error::InputError;
use lykiadb_lang::ast::{Identifier, Span};
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum PlannerError {
    #[error("Nested aggregation is not allowed")]
    NestedAggregationNotAllowed(Span),

    #[error("Aggregation is not allowed in {1}")]
    AggregationNotAllowed(Span, String),

    #[error("HAVING clause without aggregation is not allowed")]
    HavingWithoutAggregationNotAllowed(Span),

    #[error("Subquery not allowed in this context")]
    SubqueryNotAllowed(Span),

    #[error("Object '{0}' not found in scope")]
    ObjectNotFoundInScope(Identifier),

    #[error("Duplicate object '{0}' in scope")]
    DuplicateObjectInScope(Identifier),

    #[error("SELECT * with aggregation is not allowed")]
    SelectAllWithAggregationNotAllowed(Span),
}

impl From<PlannerError> for InputError {
    fn from(value: PlannerError) -> Self {
        let (hint, sp) = match &value {
            PlannerError::NestedAggregationNotAllowed(span) => {
                ("Remove the nested aggregation", *span)
            }
            PlannerError::AggregationNotAllowed(span, _) => ("Remove aggregation", *span),
            PlannerError::HavingWithoutAggregationNotAllowed(span) => {
                ("Add aggregation or remove HAVING clause", *span)
            }
            PlannerError::SubqueryNotAllowed(span) => ("Remove subquery", *span),
            PlannerError::ObjectNotFoundInScope(ident) => (
                "Check if the object is properly defined in scope",
                ident.span,
            ),
            PlannerError::DuplicateObjectInScope(ident) => (
                "Make sure object names are unique within the same scope",
                ident.span,
            ),
            PlannerError::SelectAllWithAggregationNotAllowed(span) => (
                "Specify explicit projections instead of using SELECT *",
                *span,
            ),
        };

        InputError::new(&value.to_string(), hint, Some(sp.into()))
    }
}
