//! Auto-rule repository — SQL operations for the `auto_rules` table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DbError;

/// Row type matching the `auto_rules` table schema.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AutoRuleRow {
    /// Rule ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Rule name.
    pub name: String,
    /// Execution priority (lower = higher priority).
    pub priority: i32,
    /// Whether the rule is active.
    pub is_enabled: bool,
    /// Rule conditions (JSONB array).
    pub conditions: serde_json::Value,
    /// Rule actions (JSONB array).
    pub actions: serde_json::Value,
    /// Metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: time::OffsetDateTime,
    /// Last update timestamp.
    pub updated_at: time::OffsetDateTime,
}

/// List auto-rules for a user, ordered by priority.
pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<AutoRuleRow>, DbError> {
    let rows = sqlx::query_as::<_, AutoRuleRow>(
        "SELECT id, user_id, name, priority, is_enabled,
                conditions, actions, metadata, created_at, updated_at
         FROM auto_rules WHERE user_id = $1
         ORDER BY priority, created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Find a rule by ID (owned by user).
pub async fn find_by_id(
    pool: &PgPool,
    user_id: Uuid,
    rule_id: Uuid,
) -> Result<AutoRuleRow, DbError> {
    sqlx::query_as::<_, AutoRuleRow>(
        "SELECT id, user_id, name, priority, is_enabled,
                conditions, actions, metadata, created_at, updated_at
         FROM auto_rules WHERE id = $1 AND user_id = $2",
    )
    .bind(rule_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Insert a new auto-rule.
pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    priority: i32,
    conditions: &serde_json::Value,
    actions: &serde_json::Value,
) -> Result<AutoRuleRow, DbError> {
    sqlx::query_as::<_, AutoRuleRow>(
        "INSERT INTO auto_rules (user_id, name, priority, conditions, actions)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, user_id, name, priority, is_enabled,
                   conditions, actions, metadata, created_at, updated_at",
    )
    .bind(user_id)
    .bind(name)
    .bind(priority)
    .bind(conditions)
    .bind(actions)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            DbError::UniqueViolation("rule name".into())
        }
        _ => DbError::Sqlx(e),
    })
}

/// Update a rule.
#[expect(clippy::too_many_arguments)]
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    rule_id: Uuid,
    name: Option<&str>,
    priority: Option<i32>,
    is_enabled: Option<bool>,
    conditions: Option<&serde_json::Value>,
    actions: Option<&serde_json::Value>,
) -> Result<AutoRuleRow, DbError> {
    sqlx::query_as::<_, AutoRuleRow>(
        "UPDATE auto_rules
         SET name = COALESCE($3, name),
             priority = COALESCE($4, priority),
             is_enabled = COALESCE($5, is_enabled),
             conditions = COALESCE($6, conditions),
             actions = COALESCE($7, actions)
         WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, name, priority, is_enabled,
                   conditions, actions, metadata, created_at, updated_at",
    )
    .bind(rule_id)
    .bind(user_id)
    .bind(name)
    .bind(priority)
    .bind(is_enabled)
    .bind(conditions)
    .bind(actions)
    .fetch_optional(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            DbError::UniqueViolation("rule name".into())
        }
        _ => DbError::Sqlx(e),
    })?
    .ok_or(DbError::NotFound)
}

/// Delete a rule.
pub async fn delete(pool: &PgPool, user_id: Uuid, rule_id: Uuid) -> Result<(), DbError> {
    let result = sqlx::query("DELETE FROM auto_rules WHERE id = $1 AND user_id = $2")
        .bind(rule_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}
