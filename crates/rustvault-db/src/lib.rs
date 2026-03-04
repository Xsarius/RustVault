//! RustVault database layer.
//!
//! Provides the repository pattern over PostgreSQL via SQLx,
//! connection pool management, and embedded migrations.

#![warn(missing_docs)]

pub mod error;
pub mod pool;
pub mod repos;

pub use error::DbError;
pub use pool::create_pool;

/// Result type alias for database operations.
pub type DbResult<T> = Result<T, DbError>;
