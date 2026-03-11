//! User repository — SQL operations for the `users` table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DbError;

/// Row type matching the `users` table schema.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    /// User ID.
    pub id: Uuid,
    /// Display name.
    pub username: String,
    /// Email address.
    pub email: String,
    /// Argon2id password hash (None for OIDC-only users).
    pub password_hash: Option<String>,
    /// User role.
    pub role: String,
    /// Authentication provider.
    pub auth_provider: String,
    /// OIDC subject identifier.
    pub oidc_subject: Option<String>,
    /// OIDC issuer URL.
    pub oidc_issuer: Option<String>,
    /// Preferred locale.
    pub locale: String,
    /// Preferred timezone.
    pub timezone: String,
    /// User settings (JSONB).
    pub settings: serde_json::Value,
    /// Creation timestamp.
    pub created_at: time::OffsetDateTime,
}

/// Find a user by ID.
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<UserRow, DbError> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, username, email, password_hash, role::text, auth_provider::text,
                oidc_subject, oidc_issuer, locale, timezone, settings, created_at
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Lightweight check: return the user's current role if the user exists.
///
/// Used by auth middleware to validate sessions without fetching the full row.
pub async fn get_role(pool: &PgPool, id: Uuid) -> Result<String, DbError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT role::text FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    row.map(|(role,)| role).ok_or(DbError::NotFound)
}

/// Find a user by email address.
pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<UserRow, DbError> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, username, email, password_hash, role::text, auth_provider::text,
                oidc_subject, oidc_issuer, locale, timezone, settings, created_at
         FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Find a user by OIDC issuer and subject.
pub async fn find_by_oidc(
    pool: &PgPool,
    issuer: &str,
    subject: &str,
) -> Result<Option<UserRow>, DbError> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, email, password_hash, role::text, auth_provider::text,
                oidc_subject, oidc_issuer, locale, timezone, settings, created_at
         FROM users WHERE oidc_issuer = $1 AND oidc_subject = $2",
    )
    .bind(issuer)
    .bind(subject)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Count total users (used to determine if this is the first registration).
pub async fn count(pool: &PgPool) -> Result<i64, DbError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(count.0)
}

/// Insert a new user (local registration).
pub async fn insert(
    pool: &PgPool,
    username: &str,
    email: &str,
    password_hash: &str,
    role: &str,
) -> Result<UserRow, DbError> {
    sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (username, email, password_hash, role)
         VALUES ($1, $2, $3, $4::user_role)
         RETURNING id, username, email, password_hash, role::text, auth_provider::text,
                   oidc_subject, oidc_issuer, locale, timezone, settings, created_at",
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(role)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            let detail = db_err.message().to_string();
            if detail.contains("email") {
                DbError::UniqueViolation("email".into())
            } else if detail.contains("username") {
                DbError::UniqueViolation("username".into())
            } else {
                DbError::UniqueViolation("user".into())
            }
        }
        _ => DbError::Sqlx(e),
    })
}

/// Insert a new OIDC user (auto-provisioned on first OIDC login).
pub async fn insert_oidc(
    pool: &PgPool,
    username: &str,
    email: &str,
    oidc_issuer: &str,
    oidc_subject: &str,
) -> Result<UserRow, DbError> {
    sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (username, email, auth_provider, oidc_issuer, oidc_subject)
         VALUES ($1, $2, 'oidc', $3, $4)
         RETURNING id, username, email, password_hash, role::text, auth_provider::text,
                   oidc_subject, oidc_issuer, locale, timezone, settings, created_at",
    )
    .bind(username)
    .bind(email)
    .bind(oidc_issuer)
    .bind(oidc_subject)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            DbError::UniqueViolation("user".into())
        }
        _ => DbError::Sqlx(e),
    })
}

/// Link an OIDC identity to an existing local user (dual-auth).
pub async fn link_oidc(
    pool: &PgPool,
    user_id: Uuid,
    oidc_issuer: &str,
    oidc_subject: &str,
) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE users SET auth_provider = 'both', oidc_issuer = $2, oidc_subject = $3
         WHERE id = $1",
    )
    .bind(user_id)
    .bind(oidc_issuer)
    .bind(oidc_subject)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update user settings (locale, timezone, settings JSON).
pub async fn update_settings(
    pool: &PgPool,
    user_id: Uuid,
    locale: Option<&str>,
    timezone: Option<&str>,
    settings: Option<&serde_json::Value>,
) -> Result<UserRow, DbError> {
    sqlx::query_as::<_, UserRow>(
        "UPDATE users
         SET locale = COALESCE($2, locale),
             timezone = COALESCE($3, timezone),
             settings = COALESCE($4, settings)
         WHERE id = $1
         RETURNING id, username, email, password_hash, role::text, auth_provider::text,
                   oidc_subject, oidc_issuer, locale, timezone, settings, created_at",
    )
    .bind(user_id)
    .bind(locale)
    .bind(timezone)
    .bind(settings)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}
