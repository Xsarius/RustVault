//! Tag service — business logic for tag CRUD operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::CoreError;
use crate::models::tag::Tag;

/// List all tags for a user.
pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<Tag>, CoreError> {
    let rows = rustvault_db::repos::tag::list_by_user(pool, user_id).await?;
    Ok(rows.into_iter().map(row_to_tag).collect())
}

/// Get a single tag by ID.
pub async fn get(pool: &PgPool, user_id: Uuid, tag_id: Uuid) -> Result<Tag, CoreError> {
    let row = rustvault_db::repos::tag::find_by_id(pool, user_id, tag_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "tag".into(),
                id: tag_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;
    Ok(row_to_tag(row))
}

/// Create a new tag.
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    color: Option<&str>,
) -> Result<Tag, CoreError> {
    let row = rustvault_db::repos::tag::insert(pool, user_id, name, color)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::UniqueViolation(_) => {
                CoreError::Conflict(format!("tag '{name}' already exists"))
            }
            other => CoreError::Db(other),
        })?;

    let new_value = serde_json::to_value(&row_to_tag(row.clone())).ok();
    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "tag",
        row.id,
        "create",
        None,
        new_value.as_ref(),
    )
    .await;

    Ok(row_to_tag(row))
}

/// Bulk create tags.
pub async fn bulk_create(
    pool: &PgPool,
    user_id: Uuid,
    tags: &[(String, Option<String>)],
) -> Result<Vec<Tag>, CoreError> {
    let rows = rustvault_db::repos::tag::bulk_insert(pool, user_id, tags).await?;
    Ok(rows.into_iter().map(row_to_tag).collect())
}

/// Update an existing tag.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    tag_id: Uuid,
    name: Option<&str>,
    color: Option<&str>,
) -> Result<Tag, CoreError> {
    let row = rustvault_db::repos::tag::update(pool, user_id, tag_id, name, color)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "tag".into(),
                id: tag_id.to_string(),
            },
            rustvault_db::DbError::UniqueViolation(_) => {
                CoreError::Conflict("tag name already exists".into())
            }
            other => CoreError::Db(other),
        })?;

    Ok(row_to_tag(row))
}

/// Delete a tag.
pub async fn delete(pool: &PgPool, user_id: Uuid, tag_id: Uuid) -> Result<(), CoreError> {
    let deleted = rustvault_db::repos::tag::delete(pool, user_id, tag_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "tag".into(),
                id: tag_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let old_value = serde_json::to_value(&row_to_tag(deleted)).ok();
    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "tag",
        tag_id,
        "delete",
        old_value.as_ref(),
        None,
    )
    .await;

    Ok(())
}

fn row_to_tag(row: rustvault_db::repos::tag::TagRow) -> Tag {
    Tag {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        color: row.color,
        created_at: row.created_at,
    }
}
