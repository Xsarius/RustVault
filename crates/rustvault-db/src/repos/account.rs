//! Account repository — SQL operations for the `accounts` table.

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DbError;

/// Row type matching the `accounts` table schema.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountRow {
    /// Account ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Parent bank ID.
    pub bank_id: Uuid,
    /// Display name.
    pub name: String,
    /// ISO 4217 currency code.
    pub currency: String,
    /// Account type (as text from enum cast).
    #[sqlx(rename = "type")]
    pub account_type: String,
    /// Cached balance.
    pub balance_cache: Decimal,
    /// Whether non-standard top-ups are supported (e.g. card payments from other accounts).
    pub supports_nonstandard_topup: bool,
    /// Whether archived.
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

/// List accounts for a user with optional filters.
pub async fn list_by_user(
    pool: &PgPool,
    user_id: Uuid,
    bank_id: Option<Uuid>,
    account_type: Option<&str>,
    currency: Option<&str>,
    include_archived: bool,
) -> Result<Vec<AccountRow>, DbError> {
    // Build dynamic query via conditional filters
    let rows = sqlx::query_as::<_, AccountRow>(
        "SELECT id, user_id, bank_id, name, currency, type::text, balance_cache,
                supports_nonstandard_topup, is_archived, sort_order, metadata,
                created_at, updated_at
         FROM accounts
         WHERE user_id = $1
           AND ($2::uuid IS NULL OR bank_id = $2)
           AND ($3::text IS NULL OR type::text = $3)
           AND ($4::text IS NULL OR currency = $4)
           AND ($5 OR NOT is_archived)
         ORDER BY sort_order, name",
    )
    .bind(user_id)
    .bind(bank_id)
    .bind(account_type)
    .bind(currency)
    .bind(include_archived)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Find an account by ID (owned by user).
pub async fn find_by_id(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
) -> Result<AccountRow, DbError> {
    sqlx::query_as::<_, AccountRow>(
        "SELECT id, user_id, bank_id, name, currency, type::text, balance_cache,
                supports_nonstandard_topup, is_archived, sort_order, metadata,
                created_at, updated_at
         FROM accounts WHERE id = $1 AND user_id = $2",
    )
    .bind(account_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Insert a new account.
pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    bank_id: Uuid,
    name: &str,
    currency: &str,
    account_type: &str,
    supports_nonstandard_topup: bool,
) -> Result<AccountRow, DbError> {
    sqlx::query_as::<_, AccountRow>(
        "INSERT INTO accounts (user_id, bank_id, name, currency, type, supports_nonstandard_topup)
         VALUES ($1, $2, $3, $4, $5::account_type, $6)
         RETURNING id, user_id, bank_id, name, currency, type::text, balance_cache,
                   supports_nonstandard_topup, is_archived, sort_order, metadata,
                   created_at, updated_at",
    )
    .bind(user_id)
    .bind(bank_id)
    .bind(name)
    .bind(currency)
    .bind(account_type)
    .bind(supports_nonstandard_topup)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            DbError::UniqueViolation("account name".into())
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            DbError::ForeignKeyViolation("bank_id".into())
        }
        _ => DbError::Sqlx(e),
    })
}

/// Update account fields.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    name: Option<&str>,
    currency: Option<&str>,
    account_type: Option<&str>,
    supports_nonstandard_topup: Option<bool>,
) -> Result<AccountRow, DbError> {
    sqlx::query_as::<_, AccountRow>(
        "UPDATE accounts
         SET name = COALESCE($3, name),
             currency = COALESCE($4, currency),
             type = COALESCE($5::account_type, type),
             supports_nonstandard_topup = COALESCE($6, supports_nonstandard_topup)
         WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, bank_id, name, currency, type::text, balance_cache,
                   supports_nonstandard_topup, is_archived, sort_order, metadata,
                   created_at, updated_at",
    )
    .bind(account_id)
    .bind(user_id)
    .bind(name)
    .bind(currency)
    .bind(account_type)
    .bind(supports_nonstandard_topup)
    .fetch_optional(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            DbError::UniqueViolation("account name".into())
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            DbError::ForeignKeyViolation("bank_id".into())
        }
        _ => DbError::Sqlx(e),
    })?
    .ok_or(DbError::NotFound)
}

/// Soft-archive an account.
pub async fn archive(pool: &PgPool, user_id: Uuid, account_id: Uuid) -> Result<(), DbError> {
    let result = sqlx::query(
        "UPDATE accounts SET is_archived = true WHERE id = $1 AND user_id = $2",
    )
    .bind(account_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}
