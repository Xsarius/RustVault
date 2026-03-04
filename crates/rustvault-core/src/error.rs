//! Core domain error types.

use thiserror::Error;

/// Errors originating from the domain/core layer.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Authentication failed (invalid credentials, expired token, etc.).
    #[error("authentication failed: {0}")]
    AuthFailed(String),

    /// Validation error (invalid input data).
    #[error("validation error: {0}")]
    Validation(String),

    /// Entity not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Conflict (e.g., duplicate entry).
    #[error("conflict: {0}")]
    Conflict(String),

    /// Forbidden action.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// Database error (propagated from rustvault-db).
    #[error(transparent)]
    Db(#[from] rustvault_db::DbError),
}
