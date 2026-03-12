//! Transfer service — business logic for transfer operations.

use rust_decimal::Decimal;
use sqlx::PgPool;
use time::Date;
use uuid::Uuid;

use crate::error::CoreError;
use crate::models::transfer::{Transfer, TransferMethod, TransferStatus, TransferSuggestion};

/// Create a new transfer between two accounts. Auto-creates linked debit + credit transactions.
#[expect(clippy::too_many_arguments)]
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    from_account_id: Uuid,
    to_account_id: Uuid,
    amount: Decimal,
    date: Date,
    description: Option<&str>,
    method: TransferMethod,
    received_amount: Option<Decimal>,
) -> Result<Transfer, CoreError> {
    if from_account_id == to_account_id {
        return Err(CoreError::Validation(
            "Source and destination accounts must be different".into(),
        ));
    }

    // Verify both accounts exist and belong to user
    let from_account = rustvault_db::repos::account::find_by_id(pool, user_id, from_account_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "account".into(),
                id: from_account_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let _to_account = rustvault_db::repos::account::find_by_id(pool, user_id, to_account_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "account".into(),
                id: to_account_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let desc = description.unwrap_or("Transfer");
    let credit_amount = received_amount.unwrap_or(amount);

    // Compute exchange rate for cross-currency transfers
    let exchange_rate = if received_amount.is_some() && amount != Decimal::ZERO {
        Some(credit_amount / amount)
    } else {
        None
    };

    // Create debit transaction (expense from source)
    let debit_row = rustvault_db::repos::transaction::insert(
        pool,
        user_id,
        from_account_id,
        None,     // category_id
        None,     // import_id
        "transfer",
        -amount.abs(), // negative for debit
        &from_account.currency,
        date,
        desc,
        None, // original_desc
        None, // payee
        None, // reference
        None, // notes
    )
    .await?;

    // Create credit transaction (income to destination)
    let credit_row = rustvault_db::repos::transaction::insert(
        pool,
        user_id,
        to_account_id,
        None,     // category_id
        None,     // import_id
        "transfer",
        credit_amount.abs(), // positive for credit
        &from_account.currency,
        date,
        desc,
        None, // original_desc
        None, // payee
        None, // reference
        None, // notes
    )
    .await?;

    // Create the transfer link
    let method_str = transfer_method_to_str(method);
    let row = rustvault_db::repos::transfer::insert(
        pool,
        user_id,
        debit_row.id,
        credit_row.id,
        method_str,
        "confirmed",
        exchange_rate,
        Some(Decimal::from(100)), // manual creation = 100% confidence
        None,
    )
    .await?;

    let transfer = row_to_transfer(row);

    let new_value = serde_json::to_value(&transfer).ok();
    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "transfer",
        transfer.id,
        "create",
        None,
        new_value.as_ref(),
    )
    .await;

    Ok(transfer)
}

/// Link two existing transactions as a transfer pair.
pub async fn link(
    pool: &PgPool,
    user_id: Uuid,
    debit_tx_id: Uuid,
    credit_tx_id: Uuid,
    method: TransferMethod,
) -> Result<Transfer, CoreError> {
    // Verify both transactions exist and belong to user
    let debit = rustvault_db::repos::transaction::find_by_id(pool, user_id, debit_tx_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "transaction".into(),
                id: debit_tx_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let credit = rustvault_db::repos::transaction::find_by_id(pool, user_id, credit_tx_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "transaction".into(),
                id: credit_tx_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    // Verify they are on different accounts
    if debit.account_id == credit.account_id {
        return Err(CoreError::Validation(
            "Linked transactions must be on different accounts".into(),
        ));
    }

    // Check neither is already part of a transfer
    if rustvault_db::repos::transfer::find_by_transaction_id(pool, user_id, debit_tx_id)
        .await?
        .is_some()
    {
        return Err(CoreError::Conflict(
            "Debit transaction is already part of a transfer".into(),
        ));
    }
    if rustvault_db::repos::transfer::find_by_transaction_id(pool, user_id, credit_tx_id)
        .await?
        .is_some()
    {
        return Err(CoreError::Conflict(
            "Credit transaction is already part of a transfer".into(),
        ));
    }

    // Compute exchange rate
    let debit_abs = debit.amount.abs();
    let credit_abs = credit.amount.abs();
    let exchange_rate = if debit_abs != Decimal::ZERO && debit_abs != credit_abs {
        Some(credit_abs / debit_abs)
    } else {
        None
    };

    let method_str = transfer_method_to_str(method);
    let row = rustvault_db::repos::transfer::insert(
        pool,
        user_id,
        debit_tx_id,
        credit_tx_id,
        method_str,
        "confirmed",
        exchange_rate,
        None,
        None,
    )
    .await
    .map_err(|e| match e {
        rustvault_db::DbError::UniqueViolation(msg) => CoreError::Conflict(msg),
        other => CoreError::Db(other),
    })?;

    // Update transaction types to "transfer"
    let _ = rustvault_db::repos::transaction::update(
        pool, user_id, debit_tx_id,
        None, Some("transfer"), None, None, None, None, None, None,
    ).await;
    let _ = rustvault_db::repos::transaction::update(
        pool, user_id, credit_tx_id,
        None, Some("transfer"), None, None, None, None, None, None,
    ).await;

    Ok(row_to_transfer(row))
}

/// Unlink a transfer (delete the transfer record; transactions remain).
pub async fn unlink(pool: &PgPool, user_id: Uuid, transfer_id: Uuid) -> Result<(), CoreError> {
    // Get the transfer to find linked transactions
    let transfer = rustvault_db::repos::transfer::find_by_id(pool, user_id, transfer_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "transfer".into(),
                id: transfer_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    rustvault_db::repos::transfer::delete(pool, user_id, transfer_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "transfer".into(),
                id: transfer_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    // Revert transaction types: debit → expense, credit → income
    let _ = rustvault_db::repos::transaction::update(
        pool, user_id, transfer.debit_tx_id,
        None, Some("expense"), None, None, None, None, None, None,
    ).await;
    let _ = rustvault_db::repos::transaction::update(
        pool, user_id, transfer.credit_tx_id,
        None, Some("income"), None, None, None, None, None, None,
    ).await;

    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "transfer",
        transfer_id,
        "delete",
        None,
        None,
    )
    .await;

    Ok(())
}

/// Detect potential transfer matches across user's accounts.
pub async fn detect(
    pool: &PgPool,
    user_id: Uuid,
    date_tolerance_days: i32,
    amount_tolerance: Decimal,
) -> Result<Vec<TransferSuggestion>, CoreError> {
    let matches = rustvault_db::repos::transfer::find_matches(
        pool,
        user_id,
        date_tolerance_days,
        amount_tolerance,
    )
    .await?;

    // Score and de-duplicate: each transaction can only appear in one suggestion
    let mut used_tx_ids = std::collections::HashSet::new();
    let mut suggestions = Vec::new();

    for m in matches {
        if used_tx_ids.contains(&m.debit_tx_id) || used_tx_ids.contains(&m.credit_tx_id) {
            continue;
        }

        let confidence = compute_confidence(
            m.debit_amount,
            m.credit_amount,
            m.debit_date,
            m.credit_date,
        );

        used_tx_ids.insert(m.debit_tx_id);
        used_tx_ids.insert(m.credit_tx_id);

        suggestions.push(TransferSuggestion {
            debit_tx_id: m.debit_tx_id,
            credit_tx_id: m.credit_tx_id,
            debit_account_id: m.debit_account_id,
            credit_account_id: m.credit_account_id,
            debit_amount: m.debit_amount,
            credit_amount: m.credit_amount,
            debit_desc: m.debit_desc,
            credit_desc: m.credit_desc,
            confidence,
        });
    }

    Ok(suggestions)
}

/// Compute confidence score for a potential transfer match.
fn compute_confidence(
    debit_amount: Decimal,
    credit_amount: Decimal,
    debit_date: Date,
    credit_date: Date,
) -> Decimal {
    let mut score = Decimal::from(100);

    // Amount difference penalty
    let amount_diff = (debit_amount - credit_amount).abs();
    if amount_diff > Decimal::ZERO {
        // Reduce confidence proportionally
        let pct = if debit_amount != Decimal::ZERO {
            amount_diff / debit_amount * Decimal::from(100)
        } else {
            Decimal::from(50)
        };
        score -= pct.min(Decimal::from(40));
    }

    // Date difference penalty (5 points per day)
    let date_diff = (debit_date - credit_date).whole_days().unsigned_abs();
    let date_penalty = Decimal::from(date_diff * 5).min(Decimal::from(25));
    score -= date_penalty;

    score.max(Decimal::ZERO)
}

fn transfer_method_to_str(m: TransferMethod) -> &'static str {
    match m {
        TransferMethod::Internal => "internal",
        TransferMethod::CardPayment => "card_payment",
        TransferMethod::Wire => "wire",
        TransferMethod::Other => "other",
    }
}

fn str_to_transfer_method(s: &str) -> TransferMethod {
    match s {
        "internal" => TransferMethod::Internal,
        "card_payment" => TransferMethod::CardPayment,
        "wire" => TransferMethod::Wire,
        _ => TransferMethod::Other,
    }
}

fn str_to_transfer_status(s: &str) -> TransferStatus {
    match s {
        "suggested" => TransferStatus::Suggested,
        "confirmed" => TransferStatus::Confirmed,
        "rejected" => TransferStatus::Rejected,
        _ => TransferStatus::Suggested,
    }
}

fn row_to_transfer(row: rustvault_db::repos::transfer::TransferRow) -> Transfer {
    Transfer {
        id: row.id,
        user_id: row.user_id,
        debit_tx_id: row.debit_tx_id,
        credit_tx_id: row.credit_tx_id,
        method: str_to_transfer_method(&row.method),
        status: str_to_transfer_status(&row.status),
        exchange_rate: row.exchange_rate,
        confidence: row.confidence,
        notes: row.notes,
        metadata: row.metadata,
        created_at: row.created_at,
    }
}
