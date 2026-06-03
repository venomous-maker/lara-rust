use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Binding not found for `{0}`")]
    BindingNotFound(String),

    #[error("Failed to resolve `{0}`: {1}")]
    ResolutionFailed(String, String),

    #[error("Circular dependency detected for `{0}`")]
    CircularDependency(String),

    #[error("Service provider error: {0}")]
    ServiceProvider(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Application error: {0}")]
    Application(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;
