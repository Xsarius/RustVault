//! Category repository — SQL operations for the `categories` table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DbError;

/// Row type matching the `categories` table schema.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CategoryRow {
    /// Category ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Display name.
    pub name: String,
    /// Parent category ID (None = root).
    pub parent_id: Option<Uuid>,
    /// Icon identifier.
    pub icon: Option<String>,
    /// Color hex code.
    pub color: Option<String>,
    /// Category type (as text from enum cast).
    pub category_type: String,
    /// Sort order.
    pub sort_order: i32,
    /// Metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: time::OffsetDateTime,
}

/// List all categories for a user (flat list; tree assembly done in service layer).
pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<CategoryRow>, DbError> {
    let rows = sqlx::query_as::<_, CategoryRow>(
        "SELECT id, user_id, name, parent_id, icon, color, category_type::TEXT, sort_order, metadata, created_at
         FROM categories WHERE user_id = $1
         ORDER BY sort_order, name",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Find a category by ID (owned by user).
pub async fn find_by_id(
    pool: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
) -> Result<CategoryRow, DbError> {
    sqlx::query_as::<_, CategoryRow>(
        "SELECT id, user_id, name, parent_id, icon, color, category_type::TEXT, sort_order, metadata, created_at
         FROM categories WHERE id = $1 AND user_id = $2",
    )
    .bind(category_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Insert a new category.
pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    parent_id: Option<Uuid>,
    icon: Option<&str>,
    color: Option<&str>,
    category_type: &str,
) -> Result<CategoryRow, DbError> {
    sqlx::query_as::<_, CategoryRow>(
        "INSERT INTO categories (user_id, name, parent_id, icon, color, category_type)
         VALUES ($1, $2, $3, $4, $5, $6::category_type)
         RETURNING id, user_id, name, parent_id, icon, color, category_type::TEXT, sort_order, metadata, created_at",
    )
    .bind(user_id)
    .bind(name)
    .bind(parent_id)
    .bind(icon)
    .bind(color)
    .bind(category_type)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            DbError::UniqueViolation("category name".into())
        }
        _ => DbError::Sqlx(e),
    })
}

/// Bulk-insert categories. Returns all created rows.
pub async fn bulk_insert(
    pool: &PgPool,
    user_id: Uuid,
    categories: &[(String, Option<Uuid>, Option<String>, Option<String>, String)],
) -> Result<Vec<CategoryRow>, DbError> {
    let mut results = Vec::with_capacity(categories.len());
    for (name, parent_id, icon, color, category_type) in categories {
        let row = insert(
            pool,
            user_id,
            name,
            *parent_id,
            icon.as_deref(),
            color.as_deref(),
            category_type,
        )
        .await?;
        results.push(row);
    }
    Ok(results)
}

/// Update a category.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
    name: Option<&str>,
    parent_id: Option<Option<Uuid>>,
    icon: Option<&str>,
    color: Option<&str>,
    category_type: Option<&str>,
) -> Result<CategoryRow, DbError> {
    // Handle the double-option for parent_id:
    // - None means "don't change"
    // - Some(None) means "set to NULL (root)"
    // - Some(Some(id)) means "set to id"
    let (update_parent, new_parent) = match parent_id {
        None => (false, None),
        Some(p) => (true, p),
    };

    sqlx::query_as::<_, CategoryRow>(
        "UPDATE categories
         SET name = COALESCE($3, name),
             parent_id = CASE WHEN $4 THEN $5 ELSE parent_id END,
             icon = COALESCE($6, icon),
             color = COALESCE($7, color),
             category_type = COALESCE($8::category_type, category_type)
         WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, name, parent_id, icon, color, category_type::TEXT, sort_order, metadata, created_at",
    )
    .bind(category_id)
    .bind(user_id)
    .bind(name)
    .bind(update_parent)
    .bind(new_parent)
    .bind(icon)
    .bind(color)
    .bind(category_type)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Delete a category. Returns the deleted row for audit purposes.
pub async fn delete(
    pool: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
) -> Result<CategoryRow, DbError> {
    sqlx::query_as::<_, CategoryRow>(
        "DELETE FROM categories WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, name, parent_id, icon, color, category_type::TEXT, sort_order, metadata, created_at",
    )
    .bind(category_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}
