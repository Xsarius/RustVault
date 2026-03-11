//! RustVault database layer.
//!
//! Provides the repository pattern over PostgreSQL via SQLx,
//! connection pool management, and embedded migrations.

#![warn(missing_docs)]

/// Database error types.
pub mod error;
/// Connection-pool factory.
pub mod pool;
/// Repository modules — per-entity SQL query functions.
pub mod repos;

pub use error::DbError;
pub use pool::create_pool;

/// Result type alias for database operations.
pub type DbResult<T> = Result<T, DbError>;
