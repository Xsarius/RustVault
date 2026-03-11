//! Tag repository — SQL operations for the `tags` table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DbError;

/// Row type matching the `tags` table schema.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TagRow {
    /// Tag ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Tag name.
    pub name: String,
    /// Color hex code.
    pub color: Option<String>,
    /// Creation timestamp.
    pub created_at: time::OffsetDateTime,
}

/// List all tags for a user.
pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<TagRow>, DbError> {
    let rows = sqlx::query_as::<_, TagRow>(
        "SELECT id, user_id, name, color, created_at
         FROM tags WHERE user_id = $1
         ORDER BY name",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Find a tag by ID (owned by user).
pub async fn find_by_id(pool: &PgPool, user_id: Uuid, tag_id: Uuid) -> Result<TagRow, DbError> {
    sqlx::query_as::<_, TagRow>(
        "SELECT id, user_id, name, color, created_at
         FROM tags WHERE id = $1 AND user_id = $2",
    )
    .bind(tag_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Insert a new tag.
pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    color: Option<&str>,
) -> Result<TagRow, DbError> {
    sqlx::query_as::<_, TagRow>(
        "INSERT INTO tags (user_id, name, color)
         VALUES ($1, $2, $3)
         RETURNING id, user_id, name, color, created_at",
    )
    .bind(user_id)
    .bind(name)
    .bind(color)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            DbError::UniqueViolation("tag name".into())
        }
        _ => DbError::Sqlx(e),
    })
}

/// Bulk-insert tags. Returns all created rows.
pub async fn bulk_insert(
    pool: &PgPool,
    user_id: Uuid,
    tags: &[(String, Option<String>)],
) -> Result<Vec<TagRow>, DbError> {
    let mut results = Vec::with_capacity(tags.len());
    for (name, color) in tags {
        let row = insert(pool, user_id, name, color.as_deref()).await?;
        results.push(row);
    }
    Ok(results)
}

/// Update a tag.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    tag_id: Uuid,
    name: Option<&str>,
    color: Option<&str>,
) -> Result<TagRow, DbError> {
    sqlx::query_as::<_, TagRow>(
        "UPDATE tags
         SET name = COALESCE($3, name),
             color = COALESCE($4, color)
         WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, name, color, created_at",
    )
    .bind(tag_id)
    .bind(user_id)
    .bind(name)
    .bind(color)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Delete a tag. Returns the deleted row for audit purposes.
pub async fn delete(pool: &PgPool, user_id: Uuid, tag_id: Uuid) -> Result<TagRow, DbError> {
    sqlx::query_as::<_, TagRow>(
        "DELETE FROM tags WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, name, color, created_at",
    )
    .bind(tag_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}
