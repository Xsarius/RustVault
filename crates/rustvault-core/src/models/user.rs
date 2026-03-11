//! User domain model.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// User role within the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// Full administrative access.
    Admin,
    /// Standard user access.
    Member,
    /// Read-only access.
    Viewer,
}

/// Authentication provider for the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "auth_provider", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuthProvider {
    /// Local username/password authentication.
    Local,
    /// OIDC/SSO authentication.
    Oidc,
    /// Both local and OIDC authentication.
    Both,
}

/// A registered user.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    /// Unique identifier.
    pub id: Uuid,
    /// Display name.
    pub username: String,
    /// Email address (unique).
    pub email: String,
    /// Argon2id password hash (None for OIDC-only users).
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    /// User role.
    pub role: UserRole,
    /// Authentication provider.
    pub auth_provider: AuthProvider,
    /// OIDC subject identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oidc_subject: Option<String>,
    /// OIDC issuer URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oidc_issuer: Option<String>,
    /// Preferred locale (BCP 47).
    pub locale: String,
    /// Preferred timezone (IANA).
    pub timezone: String,
    /// User-specific settings (JSON).
    pub settings: serde_json::Value,
    /// Account creation timestamp.
    pub created_at: OffsetDateTime,
}

/// Data required to register a new user.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct NewUser {
    /// Username (2–50 chars).
    #[validate(length(min = 2, max = 50))]
    pub username: String,
    /// Email address.
    #[validate(email)]
    pub email: String,
    /// Plain-text password (validated, then hashed before storage).
    #[validate(length(min = 10, max = 128))]
    pub password: String,
}

/// Login request body.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct LoginRequest {
    /// Email address.
    #[validate(email)]
    pub email: String,
    /// Plain-text password.
    #[validate(length(min = 1))]
    pub password: String,
}

/// Public-facing user info (no password hash, no internal fields).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserInfo {
    /// User ID.
    pub id: Uuid,
    /// Display name.
    pub username: String,
    /// Email address.
    pub email: String,
    /// User role.
    pub role: UserRole,
    /// Authentication provider.
    pub auth_provider: AuthProvider,
    /// Preferred locale.
    pub locale: String,
    /// Preferred timezone.
    pub timezone: String,
    /// User settings.
    pub settings: serde_json::Value,
    /// Account creation timestamp.
    pub created_at: OffsetDateTime,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            auth_provider: user.auth_provider,
            locale: user.locale,
            timezone: user.timezone,
            settings: user.settings,
            created_at: user.created_at,
        }
    }
}
