use std::collections::HashMap;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidatorError {
    #[error("Validation failed")]
    ValidationFailed(ValidationErrors),

    #[error("Rule error: {0}")]
    Rule(String),
}

/// Map of field → list of error messages.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ValidationErrors {
    pub errors: HashMap<String, Vec<String>>,
}

impl ValidationErrors {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, field: &str, message: impl Into<String>) {
        self.errors.entry(field.to_string()).or_default().push(message.into());
    }

    pub fn is_empty(&self) -> bool { self.errors.is_empty() }

    pub fn into_error(self) -> ValidatorError {
        ValidatorError::ValidationFailed(self)
    }
}

pub type ValidateResult<T> = Result<T, ValidatorError>;
