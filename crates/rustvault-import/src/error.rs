//! Import error types.

use thiserror::Error;

/// Errors originating from the import/parsing layer.
#[derive(Debug, Error)]
pub enum ImportError {
    /// Unsupported file format.
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    /// File parsing failed.
    #[error("parse error: {0}")]
    ParseFailed(String),

    /// Column mapping is required but not provided.
    #[error("column mapping required: {0}")]
    MappingRequired(String),

    /// File validation error (size, MIME type, etc.).
    #[error("file validation error: {0}")]
    FileValidation(String),

    /// I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
