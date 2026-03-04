//! RustVault AI module.
//!
//! Provides optional AI-powered features: receipt scanning, smart categorization,
//! and payee normalization. Supports multiple providers (Ollama, OpenAI, Anthropic,
//! and OpenAI-compatible endpoints).
//!
//! This module is disabled by default and can be toggled in user settings.

#![warn(missing_docs)]

pub mod error;

pub use error::AiError;

/// Result type alias for AI operations.
pub type AiResult<T> = Result<T, AiError>;
