//! RustVault HTTP server — library root.
//!
//! Re-exports modules so they can be used by integration tests and the binary.

#![warn(missing_docs)]

/// Application builder.
pub mod app;
/// Configuration loading.
pub mod config;
/// OpenAPI specification (utoipa + Scalar UI).
pub mod doc;
/// Request extractors.
pub mod extractors;
/// Middleware layers.
pub mod middleware;
/// Response types and error mapping.
pub mod response;
/// Route handlers.
pub mod routes;
/// Shared application state.
pub mod state;
