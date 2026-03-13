//! RustVault import engine.
//!
//! Provides file parsers for bank statement formats (CSV, MT940, OFX, QIF,
//! CAMT.053, XLSX, JSON, PDF), format detection, column mapping, and
//! duplicate detection.

#![warn(missing_docs)]

pub mod detect;
pub mod error;
pub mod parsers;
pub mod raw;
pub mod registry;

pub use detect::{FileFormat, detect_format};
pub use error::ImportError;
pub use raw::{ColumnMapping, ImportParser, RawTransaction};
pub use registry::ParserRegistry;

/// Result type alias for import operations.
pub type ImportResult<T> = Result<T, ImportError>;
