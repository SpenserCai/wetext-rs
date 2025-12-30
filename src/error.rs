//! Error types for WeText-RS

use thiserror::Error;

/// WeText error types
#[derive(Error, Debug)]
pub enum WeTextError {
    /// FST file not found
    #[error("FST file not found: {0}")]
    FstNotFound(String),

    /// Failed to load FST
    #[error("Failed to load FST: {0}")]
    FstLoadError(String),

    /// FST operation failed
    #[error("FST operation failed: {0}")]
    FstOperationError(String),

    /// Invalid language
    #[error("Invalid language: {0}")]
    InvalidLanguage(String),

    /// Invalid operator
    #[error("Invalid operator: {0}")]
    InvalidOperator(String),

    /// Token parse error
    #[error("Token parse error: {0}")]
    TokenParseError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type alias for WeText operations
pub type Result<T> = std::result::Result<T, WeTextError>;

