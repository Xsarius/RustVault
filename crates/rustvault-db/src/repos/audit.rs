//! Audit log repository — SQL operations for the `audit_log` table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DbError;

/// Row type matching the `audit_log` table schema.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuditRow {
    /// Entry ID.
    pub id: Uuid,
    /// User who performed the action.
    pub user_id: Option<Uuid>,
    /// Entity type (e.g., "bank", "account").
    pub entity_type: String,
    /// Affected entity ID.
    pub entity_id: Uuid,
    /// Action (e.g., "create", "update", "delete").
    pub action: String,
    /// Previous state.
    pub old_value: Option<serde_json::Value>,
    /// New state.
    pub new_value: Option<serde_json::Value>,
    /// Timestamp.
    pub created_at: time::OffsetDateTime,
}

/// Insert an audit log entry.
pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    entity_type: &str,
    entity_id: Uuid,
    action: &str,
    old_value: Option<&serde_json::Value>,
    new_value: Option<&serde_json::Value>,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO audit_log (user_id, entity_type, entity_id, action, old_value, new_value)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(user_id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(old_value)
    .bind(new_value)
    .execute(pool)
    .await?;
    Ok(())
}

/// Query audit entries for a specific entity.
pub async fn query_by_entity(
    pool: &PgPool,
    entity_type: &str,
    entity_id: Uuid,
) -> Result<Vec<AuditRow>, DbError> {
    let rows = sqlx::query_as::<_, AuditRow>(
        "SELECT id, user_id, entity_type, entity_id, action, old_value, new_value, created_at
         FROM audit_log
         WHERE entity_type = $1 AND entity_id = $2
         ORDER BY created_at DESC",
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
