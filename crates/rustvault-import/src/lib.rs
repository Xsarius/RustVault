//! RustVault import engine.
//!
//! Provides file parsers for bank statement formats (CSV, MT940, OFX, QIF,
//! CAMT.053, XLSX, JSON), format detection, column mapping, and
//! duplicate detection.

#![warn(missing_docs)]

pub mod error;

pub use error::ImportError;

/// Result type alias for import operations.
pub type ImportResult<T> = Result<T, ImportError>;
