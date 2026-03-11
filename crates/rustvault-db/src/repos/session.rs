//! Session repository — SQL operations for the `sessions` table.

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;

/// Row type matching the `sessions` table schema.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionRow {
    /// Session ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// SHA-256 hash of the refresh token.
    pub token_hash: String,
    /// Client user-agent.
    pub user_agent: Option<String>,
    /// Client IP address.
    pub ip_address: Option<String>,
    /// Expiration timestamp.
    pub expires_at: OffsetDateTime,
    /// Whether revoked.
    pub revoked: bool,
    /// Creation timestamp.
    pub created_at: OffsetDateTime,
}

/// Insert a new session (refresh token).
pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    user_agent: Option<&str>,
    ip_address: Option<&str>,
    expires_at: OffsetDateTime,
) -> Result<SessionRow, DbError> {
    sqlx::query_as::<_, SessionRow>(
        "INSERT INTO sessions (user_id, token_hash, user_agent, ip_address, expires_at)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, user_id, token_hash, user_agent, ip_address, expires_at, revoked, created_at",
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(user_agent)
    .bind(ip_address)
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map_err(DbError::from)
}

/// Find a valid (not revoked, not expired) session by token hash.
pub async fn find_valid_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<SessionRow, DbError> {
    sqlx::query_as::<_, SessionRow>(
        "SELECT id, user_id, token_hash, user_agent, ip_address, expires_at, revoked, created_at
         FROM sessions
         WHERE token_hash = $1 AND NOT revoked AND expires_at > now()",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Revoke a session by ID.
pub async fn revoke(pool: &PgPool, session_id: Uuid) -> Result<(), DbError> {
    sqlx::query("UPDATE sessions SET revoked = true WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Revoke all sessions for a user.
pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<(), DbError> {
    sqlx::query("UPDATE sessions SET revoked = true WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Count active sessions for a user.
pub async fn count_active(pool: &PgPool, user_id: Uuid) -> Result<i64, DbError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sessions WHERE user_id = $1 AND NOT revoked AND expires_at > now()",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Delete expired sessions (cleanup job).
pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, DbError> {
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at < now()")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
