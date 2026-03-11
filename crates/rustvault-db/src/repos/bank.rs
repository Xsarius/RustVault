//! Bank repository — SQL operations for the `banks` table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DbError;

/// Row type matching the `banks` table schema.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BankRow {
    /// Bank ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Display name.
    pub name: String,
    /// Whether the bank is archived.
    pub is_archived: bool,
    /// Sort order.
    pub sort_order: i32,
    /// Metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: time::OffsetDateTime,
    /// Last update timestamp.
    pub updated_at: time::OffsetDateTime,
}

/// List all banks for a user (optionally including archived).
pub async fn list_by_user(
    pool: &PgPool,
    user_id: Uuid,
    include_archived: bool,
) -> Result<Vec<BankRow>, DbError> {
    let rows = if include_archived {
        sqlx::query_as::<_, BankRow>(
            "SELECT id, user_id, name, is_archived, sort_order, metadata, created_at, updated_at
             FROM banks WHERE user_id = $1
             ORDER BY sort_order, name",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, BankRow>(
            "SELECT id, user_id, name, is_archived, sort_order, metadata, created_at, updated_at
             FROM banks WHERE user_id = $1 AND NOT is_archived
             ORDER BY sort_order, name",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
    };
    Ok(rows)
}

/// Find a bank by ID (owned by user).
pub async fn find_by_id(pool: &PgPool, user_id: Uuid, bank_id: Uuid) -> Result<BankRow, DbError> {
    sqlx::query_as::<_, BankRow>(
        "SELECT id, user_id, name, is_archived, sort_order, metadata, created_at, updated_at
         FROM banks WHERE id = $1 AND user_id = $2",
    )
    .bind(bank_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Insert a new bank.
pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
) -> Result<BankRow, DbError> {
    sqlx::query_as::<_, BankRow>(
        "INSERT INTO banks (user_id, name)
         VALUES ($1, $2)
         RETURNING id, user_id, name, is_archived, sort_order, metadata, created_at, updated_at",
    )
    .bind(user_id)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            DbError::UniqueViolation("bank name".into())
        }
        _ => DbError::Sqlx(e),
    })
}

/// Update a bank.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    bank_id: Uuid,
    name: Option<&str>,
) -> Result<BankRow, DbError> {
    sqlx::query_as::<_, BankRow>(
        "UPDATE banks
         SET name = COALESCE($3, name)
         WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, name, is_archived, sort_order, metadata, created_at, updated_at",
    )
    .bind(bank_id)
    .bind(user_id)
    .bind(name)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Archive a bank and cascade to all its accounts.
pub async fn archive(pool: &PgPool, user_id: Uuid, bank_id: Uuid) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;

    // Archive the bank
    let result = sqlx::query(
        "UPDATE banks SET is_archived = true WHERE id = $1 AND user_id = $2",
    )
    .bind(bank_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }

    // Cascade: archive all accounts belonging to this bank
    sqlx::query(
        "UPDATE accounts SET is_archived = true WHERE bank_id = $1 AND user_id = $2",
    )
    .bind(bank_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
