//! Session and authentication token models.

use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

/// A refresh token session stored server-side.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Session {
    /// Session ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// SHA-256 hash of the refresh token.
    #[serde(skip_serializing)]
    pub token_hash: String,
    /// Client user-agent string.
    pub user_agent: Option<String>,
    /// Client IP address.
    pub ip_address: Option<String>,
    /// When the session / refresh token expires.
    pub expires_at: OffsetDateTime,
    /// Whether the session has been revoked.
    pub revoked: bool,
    /// When the session was created.
    pub created_at: OffsetDateTime,
}

/// JWT claims for access tokens.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AccessTokenClaims {
    /// Subject (user ID).
    pub sub: String,
    /// User role.
    pub role: String,
    /// Issued at (Unix timestamp).
    pub iat: i64,
    /// Expiration (Unix timestamp).
    pub exp: i64,
}

/// Authentication response returned after login/register/refresh.
#[derive(Debug, Serialize)]
pub struct AuthTokens {
    /// Short-lived access token (JWT).
    pub access_token: String,
    /// Token type ("Bearer").
    pub token_type: String,
    /// Access token TTL in seconds.
    pub expires_in: u64,
}
