//! RustVault domain core.
//!
//! Contains domain models, business logic services, the rule engine,
//! i18n helpers, and cryptographic utilities. This crate has no HTTP awareness —
//! it is consumed by `rustvault-server` and other crates.

#![warn(missing_docs)]

/// Cryptographic utilities — password hashing (Argon2id), JWT creation/validation, refresh-token generation.
pub mod crypto;
/// Core error types shared across the domain layer.
pub mod error;
/// Internationalisation — Fluent bundle loading, locale resolution, message formatting.
pub mod i18n;
/// Domain models — structs for users, banks, accounts, categories, tags, sessions, settings, and audit entries.
pub mod models;
/// Business-logic services — orchestrate repository calls and enforce domain rules.
pub mod services;

pub use error::CoreError;

/// Result type alias for core operations.
pub type CoreResult<T> = Result<T, CoreError>;
