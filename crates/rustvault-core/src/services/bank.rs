//! Bank service — business logic for bank CRUD operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::CoreError;
use crate::models::bank::Bank;

/// List all banks for a user.
pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<Bank>, CoreError> {
    let rows = rustvault_db::repos::bank::list_by_user(pool, user_id, false).await?;
    Ok(rows.into_iter().map(row_to_bank).collect())
}

/// Get a single bank by ID.
pub async fn get(pool: &PgPool, user_id: Uuid, bank_id: Uuid) -> Result<Bank, CoreError> {
    let row = rustvault_db::repos::bank::find_by_id(pool, user_id, bank_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "bank".into(),
                id: bank_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;
    Ok(row_to_bank(row))
}

/// Create a new bank.
pub async fn create(pool: &PgPool, user_id: Uuid, name: &str) -> Result<Bank, CoreError> {
    let row = rustvault_db::repos::bank::insert(pool, user_id, name)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::UniqueViolation(_) => {
                CoreError::Conflict(format!("bank '{name}' already exists"))
            }
            other => CoreError::Db(other),
        })?;

    // Audit log
    let new_value = serde_json::to_value(row_to_bank(row.clone())).ok();
    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "bank",
        row.id,
        "create",
        None,
        new_value.as_ref(),
    )
    .await;

    Ok(row_to_bank(row))
}

/// Update an existing bank.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    bank_id: Uuid,
    name: Option<&str>,
) -> Result<Bank, CoreError> {
    // Fetch old value for audit
    let old_row = rustvault_db::repos::bank::find_by_id(pool, user_id, bank_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "bank".into(),
                id: bank_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;
    let old_value = serde_json::to_value(row_to_bank(old_row)).ok();

    let row = rustvault_db::repos::bank::update(pool, user_id, bank_id, name)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "bank".into(),
                id: bank_id.to_string(),
            },
            rustvault_db::DbError::UniqueViolation(_) => {
                CoreError::Conflict("bank name already exists".into())
            }
            other => CoreError::Db(other),
        })?;

    let new_value = serde_json::to_value(row_to_bank(row.clone())).ok();
    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "bank",
        bank_id,
        "update",
        old_value.as_ref(),
        new_value.as_ref(),
    )
    .await;

    Ok(row_to_bank(row))
}

/// Archive a bank and cascade to its accounts.
pub async fn archive(pool: &PgPool, user_id: Uuid, bank_id: Uuid) -> Result<(), CoreError> {
    rustvault_db::repos::bank::archive(pool, user_id, bank_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "bank".into(),
                id: bank_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let _ =
        rustvault_db::repos::audit::insert(pool, user_id, "bank", bank_id, "archive", None, None)
            .await;

    Ok(())
}

fn row_to_bank(row: rustvault_db::repos::bank::BankRow) -> Bank {
    Bank {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        is_archived: row.is_archived,
        sort_order: row.sort_order,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
