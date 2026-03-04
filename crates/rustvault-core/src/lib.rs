//! RustVault domain core.
//!
//! Contains domain models, business logic services, the rule engine,
//! i18n helpers, and cryptographic utilities. This crate has no HTTP awareness —
//! it is consumed by `rustvault-server` and other crates.

#![warn(missing_docs)]

pub mod error;
pub mod models;
pub mod services;

pub use error::CoreError;

/// Result type alias for core operations.
pub type CoreResult<T> = Result<T, CoreError>;
