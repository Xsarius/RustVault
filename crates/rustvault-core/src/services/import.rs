//! Import service — business logic for import operations.

use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::CoreError;
use crate::models::import::{
    Import, ImportExecutionResult, ImportRowError, ImportStatus, ParsedRow,
};
use crate::services::rule_engine::{self, MatchCandidate};

/// List imports for a user.
pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<Import>, CoreError> {
    let rows = rustvault_db::repos::import::list_by_user(pool, user_id).await?;
    Ok(rows.into_iter().map(row_to_import).collect())
}

/// Get a single import by ID.
pub async fn get(pool: &PgPool, user_id: Uuid, import_id: Uuid) -> Result<Import, CoreError> {
    let row = rustvault_db::repos::import::find_by_id(pool, user_id, import_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "import".into(),
                id: import_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;
    Ok(row_to_import(row))
}

/// Create a new import record (file uploaded, pending execution).
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    file_name: &str,
    file_format: &str,
    account_id: Uuid,
) -> Result<Import, CoreError> {
    // Verify the account exists and belongs to the user.
    rustvault_db::repos::account::find_by_id(pool, user_id, account_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "account".into(),
                id: account_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let row =
        rustvault_db::repos::import::insert(pool, user_id, file_name, file_format, account_id)
            .await?;

    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "import",
        row.id,
        "create",
        None,
        Some(&serde_json::json!({
            "file_name": file_name,
            "file_format": file_format,
            "account_id": account_id,
        })),
    )
    .await;

    Ok(row_to_import(row))
}

/// Execute an import — process parsed rows into transactions.
///
/// This is the main pipeline entry-point. The caller (server layer) is
/// responsible for parsing the file via `rustvault-import` and converting
/// `RawTransaction` values into `ParsedRow` values before calling this.
pub async fn execute(
    pool: &PgPool,
    user_id: Uuid,
    import_id: Uuid,
    account_id: Uuid,
    rows: &[ParsedRow],
    skip_duplicates: bool,
) -> Result<ImportExecutionResult, CoreError> {
    // Mark import as processing.
    rustvault_db::repos::import::update_status(
        pool,
        user_id,
        import_id,
        "processing",
        i32::try_from(rows.len()).unwrap_or(i32::MAX),
        0,
        0,
        0,
        0,
        None,
    )
    .await
    .map_err(|e| match e {
        rustvault_db::DbError::NotFound => CoreError::NotFound {
            entity: "import".into(),
            id: import_id.to_string(),
        },
        other => CoreError::Db(other),
    })?;

    // Fetch account to get default currency.
    let account = rustvault_db::repos::account::find_by_id(pool, user_id, account_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "account".into(),
                id: account_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    // Fetch user's enabled auto-rules.
    let rule_rows = rustvault_db::repos::auto_rule::list_by_user(pool, user_id).await?;
    let rules: Vec<crate::models::rule::AutoRule> = rule_rows
        .into_iter()
        .map(row_to_rule)
        .filter(|r| r.is_enabled)
        .collect();

    let mut imported_count: i32 = 0;
    let mut duplicate_count: i32 = 0;
    let mut error_count: i32 = 0;
    let mut errors = Vec::new();
    let mut rules_applied: HashMap<Uuid, i32> = HashMap::new();

    for (idx, row) in rows.iter().enumerate() {
        // 1. Duplicate detection.
        if skip_duplicates {
            let dupes = rustvault_db::repos::transaction::find_duplicates(
                pool, user_id, account_id, row.date, row.amount, None,
            )
            .await?;

            if !dupes.is_empty() {
                duplicate_count += 1;
                continue;
            }
        }

        // 2. Rule engine — auto-categorize.
        let candidate = MatchCandidate {
            description: row.description.clone(),
            original_desc: None,
            payee: row.payee.clone(),
            amount: row.amount,
            account_id,
        };
        let rule_result = rule_engine::apply_rules(&rules, &candidate);

        for (rule_id, _) in &rule_result.matched_rules {
            *rules_applied.entry(*rule_id).or_insert(0) += 1;
        }

        // 3. Determine transaction type.
        let transaction_type = if row.amount >= rust_decimal::Decimal::ZERO {
            "income"
        } else {
            "expense"
        };

        // 4. Choose currency (file value or account default).
        let currency = row.currency.as_deref().unwrap_or(account.currency.as_str());

        // 5. Choose payee (rule override or parsed value).
        let payee = rule_result.payee.as_deref().or(row.payee.as_deref());

        // 6. Insert transaction.
        let tx_result = rustvault_db::repos::transaction::insert(
            pool,
            user_id,
            account_id,
            rule_result.category_id,
            Some(import_id),
            transaction_type,
            row.amount,
            currency,
            row.date,
            &row.description,
            Some(&row.description),
            payee,
            row.reference.as_deref(),
            None,
        )
        .await;

        match tx_result {
            Ok(tx_row) => {
                // 7. Set tags from rule result.
                if !rule_result.tag_ids.is_empty() {
                    if let Err(e) = rustvault_db::repos::transaction::set_tags(
                        pool,
                        tx_row.id,
                        &rule_result.tag_ids,
                    )
                    .await
                    {
                        errors.push(ImportRowError {
                            row: idx,
                            message: format!("transaction inserted but tag assignment failed: {e}"),
                        });
                    }
                }
                imported_count += 1;
            }
            Err(e) => {
                error_count += 1;
                errors.push(ImportRowError {
                    row: idx,
                    message: format!("failed to insert transaction: {e}"),
                });
            }
        }
    }

    // Determine final status.
    let status = if error_count > 0 && imported_count == 0 {
        "failed"
    } else {
        "completed"
    };

    let error_details = if errors.is_empty() {
        None
    } else {
        serde_json::to_value(&errors).ok()
    };

    let final_row = rustvault_db::repos::import::update_status(
        pool,
        user_id,
        import_id,
        status,
        i32::try_from(rows.len()).unwrap_or(i32::MAX),
        imported_count,
        0,
        duplicate_count,
        error_count,
        error_details.as_ref(),
    )
    .await?;

    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "import",
        import_id,
        "execute",
        None,
        Some(&serde_json::json!({
            "total_rows": rows.len(),
            "imported": imported_count,
            "duplicates": duplicate_count,
            "errors": error_count,
        })),
    )
    .await;

    Ok(ImportExecutionResult {
        import: row_to_import(final_row),
        imported_count,
        duplicate_count,
        error_count,
        errors,
        rules_applied,
    })
}

/// Save column mapping for an import.
pub async fn save_mapping(
    pool: &PgPool,
    user_id: Uuid,
    import_id: Uuid,
    mapping: &serde_json::Value,
) -> Result<(), CoreError> {
    rustvault_db::repos::import::save_column_mapping(pool, user_id, import_id, mapping)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "import".into(),
                id: import_id.to_string(),
            },
            other => CoreError::Db(other),
        })
}

/// Rollback an import — delete all transactions from this import and mark as rolled back.
pub async fn rollback(pool: &PgPool, user_id: Uuid, import_id: Uuid) -> Result<(), CoreError> {
    // Verify the import exists and is completed
    let import = rustvault_db::repos::import::find_by_id(pool, user_id, import_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "import".into(),
                id: import_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    if import.status != "completed" {
        return Err(CoreError::Validation(format!(
            "Can only rollback completed imports (current status: {})",
            import.status
        )));
    }

    // Delete all transactions from this import
    let deleted_count =
        rustvault_db::repos::transaction::delete_by_import(pool, user_id, import_id).await?;

    // Update import status to rolled_back
    rustvault_db::repos::import::update_status(
        pool,
        user_id,
        import_id,
        "rolled_back",
        import.total_rows,
        0,
        import.skipped_count,
        import.duplicate_count,
        import.error_count,
        None,
    )
    .await?;

    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "import",
        import_id,
        "rollback",
        None,
        Some(&serde_json::json!({ "deleted_count": deleted_count })),
    )
    .await;

    Ok(())
}

fn str_to_import_status(s: &str) -> ImportStatus {
    match s {
        "pending" => ImportStatus::Pending,
        "processing" => ImportStatus::Processing,
        "completed" => ImportStatus::Completed,
        "failed" => ImportStatus::Failed,
        "rolled_back" => ImportStatus::RolledBack,
        _ => ImportStatus::Pending,
    }
}

fn row_to_import(row: rustvault_db::repos::import::ImportRow) -> Import {
    Import {
        id: row.id,
        user_id: row.user_id,
        file_name: row.file_name,
        file_format: row.file_format,
        account_id: row.account_id,
        status: str_to_import_status(&row.status),
        total_rows: row.total_rows,
        imported_count: row.imported_count,
        skipped_count: row.skipped_count,
        duplicate_count: row.duplicate_count,
        error_count: row.error_count,
        error_details: row.error_details,
        column_mapping: row.column_mapping,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn row_to_rule(row: rustvault_db::repos::auto_rule::AutoRuleRow) -> crate::models::rule::AutoRule {
    crate::models::rule::AutoRule {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        priority: row.priority,
        is_enabled: row.is_enabled,
        conditions: row.conditions,
        actions: row.actions,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
