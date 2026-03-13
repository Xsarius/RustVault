//! Transfer repository — SQL operations for the `transfers` table.

use rust_decimal::Decimal;
use sqlx::PgPool;
use time::Date;
use uuid::Uuid;

use crate::error::DbError;

/// Row type matching the `transfers` table schema.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TransferRow {
    /// Transfer ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Debit (outgoing) transaction ID.
    pub debit_tx_id: Uuid,
    /// Credit (incoming) transaction ID.
    pub credit_tx_id: Uuid,
    /// Transfer method (as text from enum cast).
    pub method: String,
    /// Transfer status (as text from enum cast).
    pub status: String,
    /// Exchange rate between currencies (nullable).
    pub exchange_rate: Option<Decimal>,
    /// Match confidence score (0–100).
    pub confidence: Option<Decimal>,
    /// User notes.
    pub notes: Option<String>,
    /// Metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: time::OffsetDateTime,
}

/// Find a transfer by ID (owned by user).
pub async fn find_by_id(
    pool: &PgPool,
    user_id: Uuid,
    transfer_id: Uuid,
) -> Result<TransferRow, DbError> {
    sqlx::query_as::<_, TransferRow>(
        "SELECT id, user_id, debit_tx_id, credit_tx_id,
                method::text, status::text, exchange_rate, confidence,
                notes, metadata, created_at
         FROM transfers WHERE id = $1 AND user_id = $2",
    )
    .bind(transfer_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Find a transfer by either transaction ID.
pub async fn find_by_transaction_id(
    pool: &PgPool,
    user_id: Uuid,
    transaction_id: Uuid,
) -> Result<Option<TransferRow>, DbError> {
    let row = sqlx::query_as::<_, TransferRow>(
        "SELECT id, user_id, debit_tx_id, credit_tx_id,
                method::text, status::text, exchange_rate, confidence,
                notes, metadata, created_at
         FROM transfers
         WHERE user_id = $1 AND (debit_tx_id = $2 OR credit_tx_id = $2)",
    )
    .bind(user_id)
    .bind(transaction_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// List all transfers for a user.
pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<TransferRow>, DbError> {
    let rows = sqlx::query_as::<_, TransferRow>(
        "SELECT id, user_id, debit_tx_id, credit_tx_id,
                method::text, status::text, exchange_rate, confidence,
                notes, metadata, created_at
         FROM transfers WHERE user_id = $1
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Insert a new transfer linking two transactions.
#[expect(clippy::too_many_arguments)]
pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    debit_tx_id: Uuid,
    credit_tx_id: Uuid,
    method: &str,
    status: &str,
    exchange_rate: Option<Decimal>,
    confidence: Option<Decimal>,
    notes: Option<&str>,
) -> Result<TransferRow, DbError> {
    sqlx::query_as::<_, TransferRow>(
        "INSERT INTO transfers (user_id, debit_tx_id, credit_tx_id, method, status,
                                exchange_rate, confidence, notes)
         VALUES ($1, $2, $3, $4::transfer_method, $5::transfer_status, $6, $7, $8)
         RETURNING id, user_id, debit_tx_id, credit_tx_id,
                   method::text, status::text, exchange_rate, confidence,
                   notes, metadata, created_at",
    )
    .bind(user_id)
    .bind(debit_tx_id)
    .bind(credit_tx_id)
    .bind(method)
    .bind(status)
    .bind(exchange_rate)
    .bind(confidence)
    .bind(notes)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            DbError::ForeignKeyViolation("transaction_id".into())
        }
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            DbError::UniqueViolation("transfer already exists".into())
        }
        sqlx::Error::Database(db_err) if db_err.is_check_violation() => {
            DbError::UniqueViolation("debit and credit transactions must be different".into())
        }
        _ => DbError::Sqlx(e),
    })
}

/// Delete a transfer (unlink transactions — transactions remain).
pub async fn delete(pool: &PgPool, user_id: Uuid, transfer_id: Uuid) -> Result<(), DbError> {
    let result = sqlx::query("DELETE FROM transfers WHERE id = $1 AND user_id = $2")
        .bind(transfer_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// A row representing a potential transfer match candidate.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TransferMatchRow {
    /// Debit transaction ID.
    pub debit_tx_id: Uuid,
    /// Credit transaction ID.
    pub credit_tx_id: Uuid,
    /// Debit account ID.
    pub debit_account_id: Uuid,
    /// Credit account ID.
    pub credit_account_id: Uuid,
    /// Debit amount (absolute value).
    pub debit_amount: Decimal,
    /// Credit amount (absolute value).
    pub credit_amount: Decimal,
    /// Debit date.
    pub debit_date: Date,
    /// Credit date.
    pub credit_date: Date,
    /// Debit description.
    pub debit_desc: String,
    /// Credit description.
    pub credit_desc: String,
}

/// Find potential transfer matches across a user's accounts.
///
/// Matches expenses on one account with income on another within
/// the given date and amount tolerances.
pub async fn find_matches(
    pool: &PgPool,
    user_id: Uuid,
    date_tolerance_days: i32,
    amount_tolerance: Decimal,
) -> Result<Vec<TransferMatchRow>, DbError> {
    let rows = sqlx::query_as::<_, TransferMatchRow>(
        "SELECT
            d.id AS debit_tx_id,
            c.id AS credit_tx_id,
            d.account_id AS debit_account_id,
            c.account_id AS credit_account_id,
            ABS(d.amount) AS debit_amount,
            ABS(c.amount) AS credit_amount,
            d.date AS debit_date,
            c.date AS credit_date,
            d.description AS debit_desc,
            c.description AS credit_desc
         FROM transactions d
         JOIN transactions c ON c.user_id = d.user_id
              AND c.account_id != d.account_id
              AND c.transaction_type::text = 'income'
              AND ABS(ABS(d.amount) - ABS(c.amount)) <= $3
              AND ABS(d.date - c.date) <= $2
              AND NOT c.is_deleted
         WHERE d.user_id = $1
           AND d.transaction_type::text = 'expense'
           AND NOT d.is_deleted
           AND NOT EXISTS (
               SELECT 1 FROM transfers t
               WHERE (t.debit_tx_id = d.id OR t.credit_tx_id = d.id)
           )
           AND NOT EXISTS (
               SELECT 1 FROM transfers t
               WHERE (t.debit_tx_id = c.id OR t.credit_tx_id = c.id)
           )
         ORDER BY ABS(ABS(d.amount) - ABS(c.amount)), ABS(d.date - c.date)
         LIMIT 100",
    )
    .bind(user_id)
    .bind(date_tolerance_days)
    .bind(amount_tolerance)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
