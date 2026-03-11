//! Account service — business logic for account CRUD operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::CoreError;
use crate::models::account::{Account, AccountType};

/// List accounts for a user with optional filters.
pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    bank_id: Option<Uuid>,
    account_type: Option<&str>,
    currency: Option<&str>,
) -> Result<Vec<Account>, CoreError> {
    let rows = rustvault_db::repos::account::list_by_user(
        pool,
        user_id,
        bank_id,
        account_type,
        currency,
        false,
    )
    .await?;

    Ok(rows.into_iter().map(row_to_account).collect())
}

/// Get a single account by ID.
pub async fn get(pool: &PgPool, user_id: Uuid, account_id: Uuid) -> Result<Account, CoreError> {
    let row = rustvault_db::repos::account::find_by_id(pool, user_id, account_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "account".into(),
                id: account_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;
    Ok(row_to_account(row))
}

/// Create a new account.
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    bank_id: Uuid,
    name: &str,
    currency: &str,
    account_type: AccountType,
    supports_nonstandard_topup: bool,
) -> Result<Account, CoreError> {
    // Verify bank exists and belongs to user
    rustvault_db::repos::bank::find_by_id(pool, user_id, bank_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "bank".into(),
                id: bank_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let type_str = account_type_to_str(account_type);
    let row = rustvault_db::repos::account::insert(
        pool,
        user_id,
        bank_id,
        name,
        currency,
        type_str,
        supports_nonstandard_topup,
    )
    .await
    .map_err(|e| match e {
        rustvault_db::DbError::UniqueViolation(_) => {
            CoreError::Conflict(format!("account '{name}' already exists"))
        }
        rustvault_db::DbError::ForeignKeyViolation(_) => CoreError::NotFound {
            entity: "bank".into(),
            id: bank_id.to_string(),
        },
        other => CoreError::Db(other),
    })?;

    let new_value = serde_json::to_value(&row_to_account(row.clone())).ok();
    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "account",
        row.id,
        "create",
        None,
        new_value.as_ref(),
    )
    .await;

    Ok(row_to_account(row))
}

/// Update an existing account.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    name: Option<&str>,
    currency: Option<&str>,
    account_type: Option<AccountType>,
    supports_nonstandard_topup: Option<bool>,
) -> Result<Account, CoreError> {
    let type_str = account_type.map(account_type_to_str);

    let row = rustvault_db::repos::account::update(
        pool,
        user_id,
        account_id,
        name,
        currency,
        type_str,
        supports_nonstandard_topup,
    )
    .await
    .map_err(|e| match e {
        rustvault_db::DbError::NotFound => CoreError::NotFound {
            entity: "account".into(),
            id: account_id.to_string(),
        },
        rustvault_db::DbError::UniqueViolation(_) => {
            CoreError::Conflict("account name already exists".into())
        }
        other => CoreError::Db(other),
    })?;

    Ok(row_to_account(row))
}

/// Archive an account.
pub async fn archive(pool: &PgPool, user_id: Uuid, account_id: Uuid) -> Result<(), CoreError> {
    rustvault_db::repos::account::archive(pool, user_id, account_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "account".into(),
                id: account_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "account",
        account_id,
        "archive",
        None,
        None,
    )
    .await;

    Ok(())
}

fn account_type_to_str(at: AccountType) -> &'static str {
    match at {
        AccountType::Checking => "checking",
        AccountType::Savings => "savings",
        AccountType::Credit => "credit",
        AccountType::Investment => "investment",
        AccountType::Loan => "loan",
    }
}

fn str_to_account_type(s: &str) -> AccountType {
    match s {
        "checking" => AccountType::Checking,
        "savings" => AccountType::Savings,
        "credit" => AccountType::Credit,
        "investment" => AccountType::Investment,
        "loan" => AccountType::Loan,
        _ => AccountType::Checking,
    }
}

fn row_to_account(row: rustvault_db::repos::account::AccountRow) -> Account {
    Account {
        id: row.id,
        user_id: row.user_id,
        bank_id: row.bank_id,
        name: row.name,
        currency: row.currency,
        account_type: str_to_account_type(&row.account_type),
        balance_cache: row.balance_cache,
        supports_nonstandard_topup: row.supports_nonstandard_topup,
        is_archived: row.is_archived,
        sort_order: row.sort_order,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
