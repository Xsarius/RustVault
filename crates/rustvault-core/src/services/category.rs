//! Category service — business logic for category CRUD operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::CoreError;
use crate::models::category::{Category, CategoryType};

/// List all categories for a user.
pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<Category>, CoreError> {
    let rows = rustvault_db::repos::category::list_by_user(pool, user_id).await?;
    Ok(rows.into_iter().map(row_to_category).collect())
}

/// Get a single category by ID.
pub async fn get(
    pool: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
) -> Result<Category, CoreError> {
    let row = rustvault_db::repos::category::find_by_id(pool, user_id, category_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "category".into(),
                id: category_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;
    Ok(row_to_category(row))
}

/// Create a new category.
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    parent_id: Option<Uuid>,
    icon: Option<&str>,
    color: Option<&str>,
    category_type: CategoryType,
) -> Result<Category, CoreError> {
    // If parent_id is set, verify it exists and belongs to user
    if let Some(pid) = parent_id {
        rustvault_db::repos::category::find_by_id(pool, user_id, pid)
            .await
            .map_err(|_| CoreError::NotFound {
                entity: "parent category".into(),
                id: pid.to_string(),
            })?;
    }

    let row =
        rustvault_db::repos::category::insert(pool, user_id, name, parent_id, icon, color, category_type.as_db_str())
            .await
            .map_err(|e| match e {
                rustvault_db::DbError::UniqueViolation(_) => {
                    CoreError::Conflict(format!("category '{name}' already exists in this context"))
                }
                other => CoreError::Db(other),
            })?;

    let new_value = serde_json::to_value(&row_to_category(row.clone())).ok();
    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "category",
        row.id,
        "create",
        None,
        new_value.as_ref(),
    )
    .await;

    Ok(row_to_category(row))
}

/// Bulk create categories.
pub async fn bulk_create(
    pool: &PgPool,
    user_id: Uuid,
    categories: &[(String, Option<Uuid>, Option<String>, Option<String>, CategoryType)],
) -> Result<Vec<Category>, CoreError> {
    let tuples: Vec<_> = categories
        .iter()
        .map(|(name, parent_id, icon, color, ct)| {
            (name.clone(), *parent_id, icon.clone(), color.clone(), ct.as_db_str().to_string())
        })
        .collect();
    let rows = rustvault_db::repos::category::bulk_insert(pool, user_id, &tuples).await?;
    Ok(rows.into_iter().map(row_to_category).collect())
}

/// Update an existing category.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
    name: Option<&str>,
    parent_id: Option<Option<Uuid>>,
    icon: Option<&str>,
    color: Option<&str>,
    category_type: Option<CategoryType>,
) -> Result<Category, CoreError> {
    let row = rustvault_db::repos::category::update(
        pool,
        user_id,
        category_id,
        name,
        parent_id,
        icon,
        color,
        category_type.map(|ct| ct.as_db_str()),
    )
    .await
    .map_err(|e| match e {
        rustvault_db::DbError::NotFound => CoreError::NotFound {
            entity: "category".into(),
            id: category_id.to_string(),
        },
        other => CoreError::Db(other),
    })?;

    Ok(row_to_category(row))
}

/// Delete a category.
pub async fn delete(pool: &PgPool, user_id: Uuid, category_id: Uuid) -> Result<(), CoreError> {
    let deleted = rustvault_db::repos::category::delete(pool, user_id, category_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "category".into(),
                id: category_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let old_value = serde_json::to_value(&row_to_category(deleted)).ok();
    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "category",
        category_id,
        "delete",
        old_value.as_ref(),
        None,
    )
    .await;

    Ok(())
}

fn row_to_category(row: rustvault_db::repos::category::CategoryRow) -> Category {
    Category {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        parent_id: row.parent_id,
        icon: row.icon,
        color: row.color,
        category_type: CategoryType::from_db(&row.category_type),
        sort_order: row.sort_order,
        metadata: row.metadata,
        created_at: row.created_at,
    }
}
