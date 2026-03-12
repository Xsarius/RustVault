//! Transaction repository — SQL operations for the `transactions` table.

use rust_decimal::Decimal;
use sqlx::PgPool;
use time::Date;
use uuid::Uuid;

use crate::error::DbError;

/// Row type matching the `transactions` table schema.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TransactionRow {
    /// Transaction ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Account this transaction belongs to.
    pub account_id: Uuid,
    /// Category (nullable).
    pub category_id: Option<Uuid>,
    /// Import session that created this transaction (nullable).
    pub import_id: Option<Uuid>,
    /// Transaction type (as text from enum cast).
    pub transaction_type: String,
    /// Amount (positive for income, negative for expense).
    pub amount: Decimal,
    /// ISO 4217 currency code.
    pub currency: String,
    /// Transaction date.
    pub date: Date,
    /// User-facing description.
    pub description: String,
    /// Original description from bank import.
    pub original_desc: Option<String>,
    /// Payee / merchant name.
    pub payee: Option<String>,
    /// Bank reference / check number.
    pub reference: Option<String>,
    /// User notes.
    pub notes: Option<String>,
    /// Whether the user has reviewed this transaction.
    pub is_reviewed: bool,
    /// Soft-delete flag.
    pub is_deleted: bool,
    /// Duplicate flag.
    pub is_duplicate: bool,
    /// Metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: time::OffsetDateTime,
    /// Last update timestamp.
    pub updated_at: time::OffsetDateTime,
}

/// Parameters for filtering transactions.
#[derive(Debug, Default)]
pub struct TransactionFilter {
    /// Filter by account.
    pub account_id: Option<Uuid>,
    /// Filter by category.
    pub category_id: Option<Uuid>,
    /// Filter by transaction type (income/expense/transfer).
    pub transaction_type: Option<String>,
    /// Filter from date (inclusive).
    pub date_from: Option<Date>,
    /// Filter to date (inclusive).
    pub date_to: Option<Date>,
    /// Full-text search query.
    pub search: Option<String>,
    /// Filter by reviewed status.
    pub is_reviewed: Option<bool>,
    /// Whether to include deleted transactions.
    pub include_deleted: bool,
    /// Filter by tag ID (transactions that have this tag).
    pub tag_id: Option<Uuid>,
    /// Filter by import ID.
    pub import_id: Option<Uuid>,
}

/// List transactions for a user with filters, cursor-based pagination.
pub async fn list_by_user(
    pool: &PgPool,
    user_id: Uuid,
    filter: &TransactionFilter,
    limit: i64,
    cursor_date: Option<Date>,
    cursor_id: Option<Uuid>,
) -> Result<Vec<TransactionRow>, DbError> {
    let rows = sqlx::query_as::<_, TransactionRow>(
        "SELECT t.id, t.user_id, t.account_id, t.category_id, t.import_id,
                t.transaction_type::text, t.amount, t.currency, t.date,
                t.description, t.original_desc, t.payee, t.reference, t.notes,
                t.is_reviewed, t.is_deleted, t.is_duplicate,
                t.metadata, t.created_at, t.updated_at
         FROM transactions t
         WHERE t.user_id = $1
           AND ($2::uuid IS NULL OR t.account_id = $2)
           AND ($3::uuid IS NULL OR t.category_id = $3)
           AND ($4::text IS NULL OR t.transaction_type::text = $4)
           AND ($5::date IS NULL OR t.date >= $5)
           AND ($6::date IS NULL OR t.date <= $6)
           AND ($7::text IS NULL OR t.search_vector @@ plainto_tsquery('simple', $7))
           AND ($8::boolean IS NULL OR t.is_reviewed = $8)
           AND ($9 OR NOT t.is_deleted)
           AND ($10::uuid IS NULL OR EXISTS (
               SELECT 1 FROM transaction_tags tt WHERE tt.transaction_id = t.id AND tt.tag_id = $10
           ))
           AND ($11::uuid IS NULL OR t.import_id = $11)
           AND (
               ($12::date IS NULL AND $13::uuid IS NULL)
               OR (t.date < $12)
               OR (t.date = $12 AND t.id < $13)
           )
         ORDER BY t.date DESC, t.id DESC
         LIMIT $14",
    )
    .bind(user_id)
    .bind(filter.account_id)
    .bind(filter.category_id)
    .bind(filter.transaction_type.as_deref())
    .bind(filter.date_from)
    .bind(filter.date_to)
    .bind(filter.search.as_deref())
    .bind(filter.is_reviewed)
    .bind(filter.include_deleted)
    .bind(filter.tag_id)
    .bind(filter.import_id)
    .bind(cursor_date)
    .bind(cursor_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Find a transaction by ID (owned by user).
pub async fn find_by_id(
    pool: &PgPool,
    user_id: Uuid,
    transaction_id: Uuid,
) -> Result<TransactionRow, DbError> {
    sqlx::query_as::<_, TransactionRow>(
        "SELECT id, user_id, account_id, category_id, import_id,
                transaction_type::text, amount, currency, date,
                description, original_desc, payee, reference, notes,
                is_reviewed, is_deleted, is_duplicate,
                metadata, created_at, updated_at
         FROM transactions WHERE id = $1 AND user_id = $2",
    )
    .bind(transaction_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Insert a new transaction.
#[expect(clippy::too_many_arguments)]
pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    category_id: Option<Uuid>,
    import_id: Option<Uuid>,
    transaction_type: &str,
    amount: Decimal,
    currency: &str,
    date: Date,
    description: &str,
    original_desc: Option<&str>,
    payee: Option<&str>,
    reference: Option<&str>,
    notes: Option<&str>,
) -> Result<TransactionRow, DbError> {
    sqlx::query_as::<_, TransactionRow>(
        "INSERT INTO transactions (
            user_id, account_id, category_id, import_id,
            transaction_type, amount, currency, date,
            description, original_desc, payee, reference, notes
         )
         VALUES ($1, $2, $3, $4, $5::transaction_type, $6, $7, $8, $9, $10, $11, $12, $13)
         RETURNING id, user_id, account_id, category_id, import_id,
                   transaction_type::text, amount, currency, date,
                   description, original_desc, payee, reference, notes,
                   is_reviewed, is_deleted, is_duplicate,
                   metadata, created_at, updated_at",
    )
    .bind(user_id)
    .bind(account_id)
    .bind(category_id)
    .bind(import_id)
    .bind(transaction_type)
    .bind(amount)
    .bind(currency)
    .bind(date)
    .bind(description)
    .bind(original_desc)
    .bind(payee)
    .bind(reference)
    .bind(notes)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            DbError::ForeignKeyViolation("account_id or category_id".into())
        }
        _ => DbError::Sqlx(e),
    })
}

/// Update a transaction.
#[expect(clippy::too_many_arguments)]
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    transaction_id: Uuid,
    category_id: Option<Option<Uuid>>,
    transaction_type: Option<&str>,
    amount: Option<Decimal>,
    date: Option<Date>,
    description: Option<&str>,
    payee: Option<Option<&str>>,
    notes: Option<Option<&str>>,
    is_reviewed: Option<bool>,
) -> Result<TransactionRow, DbError> {
    // Use a hand-constructed update to handle Option<Option<T>> for nullable fields.
    // category_id: None = no change, Some(None) = set to NULL, Some(Some(id)) = set to id.
    sqlx::query_as::<_, TransactionRow>(
        "UPDATE transactions
         SET category_id = CASE
                WHEN $3::boolean THEN $4::uuid
                ELSE category_id
             END,
             transaction_type = COALESCE($5::transaction_type, transaction_type),
             amount = COALESCE($6, amount),
             date = COALESCE($7, date),
             description = COALESCE($8, description),
             payee = CASE
                WHEN $9::boolean THEN $10::text
                ELSE payee
             END,
             notes = CASE
                WHEN $11::boolean THEN $12::text
                ELSE notes
             END,
             is_reviewed = COALESCE($13, is_reviewed)
         WHERE id = $1 AND user_id = $2 AND NOT is_deleted
         RETURNING id, user_id, account_id, category_id, import_id,
                   transaction_type::text, amount, currency, date,
                   description, original_desc, payee, reference, notes,
                   is_reviewed, is_deleted, is_duplicate,
                   metadata, created_at, updated_at",
    )
    .bind(transaction_id)
    .bind(user_id)
    .bind(category_id.is_some()) // $3: whether to update category_id
    .bind(category_id.flatten()) // $4: the new category_id value (or NULL)
    .bind(transaction_type) // $5
    .bind(amount) // $6
    .bind(date) // $7
    .bind(description) // $8
    .bind(payee.is_some()) // $9: whether to update payee
    .bind(payee.flatten()) // $10: the new payee value (or NULL)
    .bind(notes.is_some()) // $11: whether to update notes
    .bind(notes.flatten()) // $12: the new notes value (or NULL)
    .bind(is_reviewed) // $13
    .fetch_optional(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            DbError::ForeignKeyViolation("category_id".into())
        }
        _ => DbError::Sqlx(e),
    })?
    .ok_or(DbError::NotFound)
}

/// Soft-delete a transaction.
pub async fn soft_delete(
    pool: &PgPool,
    user_id: Uuid,
    transaction_id: Uuid,
) -> Result<(), DbError> {
    let result = sqlx::query(
        "UPDATE transactions SET is_deleted = true WHERE id = $1 AND user_id = $2 AND NOT is_deleted",
    )
    .bind(transaction_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Bulk update: set category, add reviewed flag for a set of transaction IDs.
pub async fn bulk_update(
    pool: &PgPool,
    user_id: Uuid,
    transaction_ids: &[Uuid],
    category_id: Option<Option<Uuid>>,
    is_reviewed: Option<bool>,
) -> Result<u64, DbError> {
    let result = sqlx::query(
        "UPDATE transactions
         SET category_id = CASE
                WHEN $3::boolean THEN $4::uuid
                ELSE category_id
             END,
             is_reviewed = COALESCE($5, is_reviewed)
         WHERE user_id = $1
           AND id = ANY($2)
           AND NOT is_deleted",
    )
    .bind(user_id)
    .bind(transaction_ids)
    .bind(category_id.is_some())
    .bind(category_id.flatten())
    .bind(is_reviewed)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Find potential duplicates: transactions with same account, date, and amount.
pub async fn find_duplicates(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    date: Date,
    amount: Decimal,
    exclude_id: Option<Uuid>,
) -> Result<Vec<TransactionRow>, DbError> {
    let rows = sqlx::query_as::<_, TransactionRow>(
        "SELECT id, user_id, account_id, category_id, import_id,
                transaction_type::text, amount, currency, date,
                description, original_desc, payee, reference, notes,
                is_reviewed, is_deleted, is_duplicate,
                metadata, created_at, updated_at
         FROM transactions
         WHERE user_id = $1
           AND account_id = $2
           AND date = $3
           AND amount = $4
           AND NOT is_deleted
           AND ($5::uuid IS NULL OR id != $5)
         ORDER BY created_at",
    )
    .bind(user_id)
    .bind(account_id)
    .bind(date)
    .bind(amount)
    .bind(exclude_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Get tag IDs for a transaction.
pub async fn get_tag_ids(
    pool: &PgPool,
    transaction_id: Uuid,
) -> Result<Vec<Uuid>, DbError> {
    let rows = sqlx::query_scalar::<_, Uuid>(
        "SELECT tag_id FROM transaction_tags WHERE transaction_id = $1",
    )
    .bind(transaction_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Set tags for a transaction (replace all existing).
pub async fn set_tags(
    pool: &PgPool,
    transaction_id: Uuid,
    tag_ids: &[Uuid],
) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM transaction_tags WHERE transaction_id = $1")
        .bind(transaction_id)
        .execute(&mut *tx)
        .await?;

    for tag_id in tag_ids {
        sqlx::query("INSERT INTO transaction_tags (transaction_id, tag_id) VALUES ($1, $2)")
            .bind(transaction_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| match &e {
                sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
                    DbError::ForeignKeyViolation("tag_id".into())
                }
                _ => DbError::Sqlx(e),
            })?;
    }

    tx.commit().await?;
    Ok(())
}

/// Add tags to transactions in bulk.
pub async fn bulk_add_tags(
    pool: &PgPool,
    user_id: Uuid,
    transaction_ids: &[Uuid],
    tag_ids: &[Uuid],
) -> Result<u64, DbError> {
    // Verify ownership first
    let mut count = 0u64;
    for tx_id in transaction_ids {
        for tag_id in tag_ids {
            let result = sqlx::query(
                "INSERT INTO transaction_tags (transaction_id, tag_id)
                 SELECT $1, $2
                 WHERE EXISTS (
                     SELECT 1 FROM transactions WHERE id = $1 AND user_id = $3 AND NOT is_deleted
                 )
                 ON CONFLICT DO NOTHING",
            )
            .bind(tx_id)
            .bind(tag_id)
            .bind(user_id)
            .execute(pool)
            .await?;
            count += result.rows_affected();
        }
    }
    Ok(count)
}

/// Delete all transactions from a specific import (rollback).
pub async fn delete_by_import(
    pool: &PgPool,
    user_id: Uuid,
    import_id: Uuid,
) -> Result<u64, DbError> {
    let result = sqlx::query(
        "DELETE FROM transactions WHERE user_id = $1 AND import_id = $2",
    )
    .bind(user_id)
    .bind(import_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
