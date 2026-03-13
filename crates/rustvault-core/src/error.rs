//! Core domain error types.

use thiserror::Error;

/// Errors originating from the domain/core layer.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Authentication failed (generic message to avoid leaking info).
    #[error("authentication failed: {reason}")]
    AuthFailed {
        /// Reason for auth failure.
        reason: String,
    },

    /// Invalid credentials (wrong password or email).
    #[error("invalid credentials")]
    InvalidCredentials,

    /// Token has expired.
    #[error("token expired")]
    TokenExpired,

    /// Token is malformed or invalid.
    #[error("token invalid: {0}")]
    TokenInvalid(String),

    /// Access denied (insufficient permissions).
    #[error("access denied")]
    AccessDenied,

    /// Account is locked due to too many failed login attempts.
    #[error("account locked")]
    AccountLocked,

    /// Validation error (invalid input data).
    #[error("validation error: {0}")]
    Validation(String),

    /// Entity not found.
    #[error("not found: {entity} with id {id}")]
    NotFound {
        /// Entity type name.
        entity: String,
        /// Entity identifier.
        id: String,
    },

    /// Conflict (e.g., duplicate entry).
    #[error("conflict: {0}")]
    Conflict(String),

    /// Forbidden action.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// OIDC authentication error.
    #[error("OIDC error: {0}")]
    OidcError(String),

    /// OIDC is not configured on this instance.
    #[error("OIDC not configured")]
    OidcNotConfigured,

    /// OIDC user not pre-registered (auto_register is off).
    #[error("OIDC user not pre-registered: {email}")]
    OidcUserNotRegistered {
        /// Email of the unregistered user.
        email: String,
    },

    /// Public registration is disabled.
    #[error("registration is disabled")]
    RegistrationDisabled,

    /// Password login not available (OIDC-only user).
    #[error("password login not available — use SSO")]
    PasswordLoginDisabled,

    /// Database error (propagated from rustvault-db).
    #[error(transparent)]
    Db(#[from] rustvault_db::DbError),

    /// External service error (e.g. ECB rate feed, OIDC provider).
    #[error("external service error: {0}")]
    ExternalService(String),

    /// Internal / unexpected error.
    #[error("internal error: {0}")]
    Internal(String),
}
