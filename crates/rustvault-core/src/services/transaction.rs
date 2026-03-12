//! Transaction service — business logic for transaction CRUD operations.

use rust_decimal::Decimal;
use sqlx::PgPool;
use time::Date;
use uuid::Uuid;

use crate::error::CoreError;
use crate::models::transaction::{Transaction, TransactionType};

/// List transactions with filters and cursor-based pagination.
#[expect(clippy::too_many_arguments)]
pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Option<Uuid>,
    category_id: Option<Uuid>,
    transaction_type: Option<&str>,
    date_from: Option<Date>,
    date_to: Option<Date>,
    search: Option<&str>,
    is_reviewed: Option<bool>,
    tag_id: Option<Uuid>,
    import_id: Option<Uuid>,
    limit: i64,
    cursor_date: Option<Date>,
    cursor_id: Option<Uuid>,
) -> Result<Vec<Transaction>, CoreError> {
    let filter = rustvault_db::repos::transaction::TransactionFilter {
        account_id,
        category_id,
        transaction_type: transaction_type.map(|s| s.to_owned()),
        date_from,
        date_to,
        search: search.map(|s| s.to_owned()),
        is_reviewed,
        include_deleted: false,
        tag_id,
        import_id,
    };

    let effective_limit = limit.clamp(1, 100);
    let rows = rustvault_db::repos::transaction::list_by_user(
        pool,
        user_id,
        &filter,
        effective_limit,
        cursor_date,
        cursor_id,
    )
    .await?;

    Ok(rows.into_iter().map(row_to_transaction).collect())
}

/// Get a single transaction by ID.
pub async fn get(
    pool: &PgPool,
    user_id: Uuid,
    transaction_id: Uuid,
) -> Result<Transaction, CoreError> {
    let row = rustvault_db::repos::transaction::find_by_id(pool, user_id, transaction_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "transaction".into(),
                id: transaction_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let mut tx = row_to_transaction(row);
    tx.tag_ids = Some(
        rustvault_db::repos::transaction::get_tag_ids(pool, transaction_id).await?,
    );
    Ok(tx)
}

/// Create a new transaction.
#[expect(clippy::too_many_arguments)]
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    category_id: Option<Uuid>,
    transaction_type: TransactionType,
    amount: Decimal,
    date: Date,
    description: &str,
    payee: Option<&str>,
    notes: Option<&str>,
    tag_ids: &[Uuid],
) -> Result<Transaction, CoreError> {
    // Disallow creating transfer-type transactions directly
    if transaction_type == TransactionType::Transfer {
        return Err(CoreError::Validation(
            "Use POST /api/transfers to create transfer transactions".into(),
        ));
    }

    // Verify account exists and belongs to user
    rustvault_db::repos::account::find_by_id(pool, user_id, account_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "account".into(),
                id: account_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let type_str = transaction_type_to_str(transaction_type);
    let row = rustvault_db::repos::transaction::insert(
        pool,
        user_id,
        account_id,
        category_id,
        None, // import_id
        type_str,
        amount,
        "", // currency will come from account
        date,
        description,
        None, // original_desc
        payee,
        None, // reference
        notes,
    )
    .await
    .map_err(|e| match e {
        rustvault_db::DbError::ForeignKeyViolation(ref field) => {
            if field.contains("category") {
                CoreError::NotFound {
                    entity: "category".into(),
                    id: category_id.map_or_else(String::new, |id| id.to_string()),
                }
            } else {
                CoreError::NotFound {
                    entity: "account".into(),
                    id: account_id.to_string(),
                }
            }
        }
        other => CoreError::Db(other),
    })?;

    // Set tags
    if !tag_ids.is_empty() {
        rustvault_db::repos::transaction::set_tags(pool, row.id, tag_ids)
            .await
            .map_err(|e| match e {
                rustvault_db::DbError::ForeignKeyViolation(_) => {
                    CoreError::Validation("One or more tag IDs are invalid".into())
                }
                other => CoreError::Db(other),
            })?;
    }

    let new_value = serde_json::to_value(row_to_transaction(row.clone())).ok();
    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "transaction",
        row.id,
        "create",
        None,
        new_value.as_ref(),
    )
    .await;

    let mut tx = row_to_transaction(row);
    tx.tag_ids = Some(tag_ids.to_vec());
    Ok(tx)
}

/// Update an existing transaction.
#[expect(clippy::too_many_arguments)]
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    transaction_id: Uuid,
    category_id: Option<Option<Uuid>>,
    transaction_type: Option<TransactionType>,
    amount: Option<Decimal>,
    date: Option<Date>,
    description: Option<&str>,
    payee: Option<Option<&str>>,
    notes: Option<Option<&str>>,
    is_reviewed: Option<bool>,
    tag_ids: Option<&[Uuid]>,
) -> Result<Transaction, CoreError> {
    let type_str = transaction_type.map(transaction_type_to_str);

    let row = rustvault_db::repos::transaction::update(
        pool,
        user_id,
        transaction_id,
        category_id,
        type_str,
        amount,
        date,
        description,
        payee,
        notes,
        is_reviewed,
    )
    .await
    .map_err(|e| match e {
        rustvault_db::DbError::NotFound => CoreError::NotFound {
            entity: "transaction".into(),
            id: transaction_id.to_string(),
        },
        rustvault_db::DbError::ForeignKeyViolation(_) => CoreError::Validation(
            "Invalid category_id".into(),
        ),
        other => CoreError::Db(other),
    })?;

    if let Some(tags) = tag_ids {
        rustvault_db::repos::transaction::set_tags(pool, transaction_id, tags)
            .await
            .map_err(|e| match e {
                rustvault_db::DbError::ForeignKeyViolation(_) => {
                    CoreError::Validation("One or more tag IDs are invalid".into())
                }
                other => CoreError::Db(other),
            })?;
    }

    let mut tx = row_to_transaction(row);
    tx.tag_ids = Some(
        rustvault_db::repos::transaction::get_tag_ids(pool, transaction_id).await?,
    );
    Ok(tx)
}

/// Soft-delete a transaction.
pub async fn delete(
    pool: &PgPool,
    user_id: Uuid,
    transaction_id: Uuid,
) -> Result<(), CoreError> {
    // If part of a transfer, unlink it first
    if let Some(transfer) =
        rustvault_db::repos::transfer::find_by_transaction_id(pool, user_id, transaction_id)
            .await?
    {
        rustvault_db::repos::transfer::delete(pool, user_id, transfer.id).await?;
    }

    rustvault_db::repos::transaction::soft_delete(pool, user_id, transaction_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "transaction".into(),
                id: transaction_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "transaction",
        transaction_id,
        "delete",
        None,
        None,
    )
    .await;

    Ok(())
}

/// Bulk update transactions.
pub async fn bulk_update(
    pool: &PgPool,
    user_id: Uuid,
    transaction_ids: &[Uuid],
    category_id: Option<Option<Uuid>>,
    is_reviewed: Option<bool>,
    add_tag_ids: &[Uuid],
) -> Result<u64, CoreError> {
    let updated = rustvault_db::repos::transaction::bulk_update(
        pool,
        user_id,
        transaction_ids,
        category_id,
        is_reviewed,
    )
    .await?;

    if !add_tag_ids.is_empty() {
        rustvault_db::repos::transaction::bulk_add_tags(
            pool,
            user_id,
            transaction_ids,
            add_tag_ids,
        )
        .await?;
    }

    Ok(updated)
}

/// Find potential duplicate transactions.
pub async fn find_duplicates(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    date: Date,
    amount: Decimal,
    exclude_id: Option<Uuid>,
) -> Result<Vec<Transaction>, CoreError> {
    let rows = rustvault_db::repos::transaction::find_duplicates(
        pool, user_id, account_id, date, amount, exclude_id,
    )
    .await?;

    Ok(rows.into_iter().map(row_to_transaction).collect())
}

fn transaction_type_to_str(tt: TransactionType) -> &'static str {
    match tt {
        TransactionType::Income => "income",
        TransactionType::Expense => "expense",
        TransactionType::Transfer => "transfer",
    }
}

fn str_to_transaction_type(s: &str) -> TransactionType {
    match s {
        "income" => TransactionType::Income,
        "expense" => TransactionType::Expense,
        "transfer" => TransactionType::Transfer,
        _ => TransactionType::Expense,
    }
}

fn row_to_transaction(row: rustvault_db::repos::transaction::TransactionRow) -> Transaction {
    Transaction {
        id: row.id,
        user_id: row.user_id,
        account_id: row.account_id,
        category_id: row.category_id,
        import_id: row.import_id,
        transaction_type: str_to_transaction_type(&row.transaction_type),
        amount: row.amount,
        currency: row.currency,
        date: row.date,
        description: row.description,
        original_desc: row.original_desc,
        payee: row.payee,
        reference: row.reference,
        notes: row.notes,
        is_reviewed: row.is_reviewed,
        is_deleted: row.is_deleted,
        is_duplicate: row.is_duplicate,
        metadata: row.metadata,
        tag_ids: None,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
