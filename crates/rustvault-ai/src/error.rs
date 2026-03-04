//! AI module error types.

use thiserror::Error;

/// Errors originating from the AI module.
#[derive(Debug, Error)]
pub enum AiError {
    /// AI provider is unavailable or not configured.
    #[error("provider unavailable: {0}")]
    ProviderUnavailable(String),

    /// AI provider returned an invalid or unparseable response.
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    /// Rate limited by the AI provider.
    #[error("rate limited: {0}")]
    RateLimited(String),

    /// HTTP request to the provider failed.
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}
