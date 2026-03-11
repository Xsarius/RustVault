//! Account domain model.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Type of financial account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "account_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// Current / checking account.
    Checking,
    /// Savings account.
    Savings,
    /// Credit card.
    Credit,
    /// Investment / brokerage account.
    Investment,
    /// Loan or mortgage.
    Loan,
}

/// A financial account belonging to a bank.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Account {
    /// Unique identifier.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Parent bank ID.
    pub bank_id: Uuid,
    /// Display name.
    pub name: String,
    /// ISO 4217 currency code.
    pub currency: String,
    /// Account type.
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub account_type: AccountType,
    /// Cached balance (sum of transactions).
    pub balance_cache: Decimal,
    /// Whether this account supports non-standard top-up methods (e.g. card payments).
    pub supports_nonstandard_topup: bool,
    /// Whether the account is soft-archived.
    pub is_archived: bool,
    /// Display sort order.
    pub sort_order: i32,
    /// Extensible metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: OffsetDateTime,
    /// Last update timestamp.
    pub updated_at: OffsetDateTime,
}

/// Data required to create a new account.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct NewAccount {
    /// Parent bank ID.
    pub bank_id: Uuid,
    /// Account name (1–100 chars).
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    /// ISO 4217 currency code.
    #[validate(length(min = 3, max = 3))]
    pub currency: String,
    /// Account type.
    #[serde(rename = "type")]
    pub account_type: AccountType,
    /// Whether this account supports non-standard top-up methods.
    #[serde(default)]
    pub supports_nonstandard_topup: bool,
}

/// Data for updating an existing account.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct UpdateAccount {
    /// Updated name.
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    /// Updated currency.
    #[validate(length(min = 3, max = 3))]
    pub currency: Option<String>,
    /// Updated account type.
    #[serde(rename = "type")]
    pub account_type: Option<AccountType>,
    /// Updated non-standard topup support.
    pub supports_nonstandard_topup: Option<bool>,
}
