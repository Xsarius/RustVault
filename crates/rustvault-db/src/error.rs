//! Database error types.

use thiserror::Error;

/// Errors originating from the database layer.
#[derive(Debug, Error)]
pub enum DbError {
    /// Entity not found.
    #[error("entity not found")]
    NotFound,

    /// Unique constraint violation (e.g., duplicate email).
    #[error("unique constraint violation: {0}")]
    UniqueViolation(String),

    /// Foreign key constraint violation.
    #[error("foreign key violation: {0}")]
    ForeignKeyViolation(String),

    /// Underlying SQLx error.
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    /// Migration error.
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
}
