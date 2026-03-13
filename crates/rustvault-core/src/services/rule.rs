//! Auto-rule service — business logic for auto-categorization rule CRUD.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::CoreError;
use crate::models::rule::AutoRule;

/// List rules for a user.
pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<AutoRule>, CoreError> {
    let rows = rustvault_db::repos::auto_rule::list_by_user(pool, user_id).await?;
    Ok(rows.into_iter().map(row_to_rule).collect())
}

/// Get a single rule by ID.
pub async fn get(pool: &PgPool, user_id: Uuid, rule_id: Uuid) -> Result<AutoRule, CoreError> {
    let row = rustvault_db::repos::auto_rule::find_by_id(pool, user_id, rule_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "rule".into(),
                id: rule_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;
    Ok(row_to_rule(row))
}

/// Create a new auto-rule.
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    priority: i32,
    conditions: &serde_json::Value,
    actions: &serde_json::Value,
) -> Result<AutoRule, CoreError> {
    let row =
        rustvault_db::repos::auto_rule::insert(pool, user_id, name, priority, conditions, actions)
            .await
            .map_err(|e| match e {
                rustvault_db::DbError::UniqueViolation(_) => {
                    CoreError::Conflict(format!("rule '{name}' already exists"))
                }
                other => CoreError::Db(other),
            })?;

    let new_value = serde_json::to_value(row_to_rule(row.clone())).ok();
    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "auto_rule",
        row.id,
        "create",
        None,
        new_value.as_ref(),
    )
    .await;

    Ok(row_to_rule(row))
}

/// Update an auto-rule.
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
) -> Result<AutoRule, CoreError> {
    let row = rustvault_db::repos::auto_rule::update(
        pool, user_id, rule_id, name, priority, is_enabled, conditions, actions,
    )
    .await
    .map_err(|e| match e {
        rustvault_db::DbError::NotFound => CoreError::NotFound {
            entity: "rule".into(),
            id: rule_id.to_string(),
        },
        rustvault_db::DbError::UniqueViolation(_) => {
            CoreError::Conflict("rule name already exists".into())
        }
        other => CoreError::Db(other),
    })?;

    Ok(row_to_rule(row))
}

/// Delete an auto-rule.
pub async fn delete(pool: &PgPool, user_id: Uuid, rule_id: Uuid) -> Result<(), CoreError> {
    rustvault_db::repos::auto_rule::delete(pool, user_id, rule_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "rule".into(),
                id: rule_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "auto_rule",
        rule_id,
        "delete",
        None,
        None,
    )
    .await;

    Ok(())
}

fn row_to_rule(row: rustvault_db::repos::auto_rule::AutoRuleRow) -> AutoRule {
    AutoRule {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        priority: row.priority,
        is_enabled: row.is_enabled,
        conditions: row.conditions,
        actions: row.actions,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
