//! Transaction domain model.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

/// Type of transaction.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "transaction_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// Income / inflow.
    Income,
    /// Expense / outflow.
    Expense,
    /// Transfer between accounts.
    Transfer,
}

/// A financial transaction.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct Transaction {
    /// Unique identifier.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Account this transaction belongs to.
    pub account_id: Uuid,
    /// Category (nullable).
    pub category_id: Option<Uuid>,
    /// Import session that created this transaction (nullable).
    pub import_id: Option<Uuid>,
    /// Transaction type.
    pub transaction_type: TransactionType,
    /// Amount.
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
    /// Extensible metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Tag IDs associated with this transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_ids: Option<Vec<Uuid>>,
    /// Creation timestamp.
    pub created_at: OffsetDateTime,
    /// Last update timestamp.
    pub updated_at: OffsetDateTime,
}

/// Data required to create a new transaction.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct NewTransaction {
    /// Account ID.
    pub account_id: Uuid,
    /// Category ID (optional).
    pub category_id: Option<Uuid>,
    /// Transaction type.
    pub transaction_type: TransactionType,
    /// Amount (positive for income, negative for expense).
    pub amount: Decimal,
    /// Transaction date.
    pub date: Date,
    /// Description.
    #[validate(length(max = 1000))]
    pub description: String,
    /// Payee / merchant name.
    #[validate(length(max = 500))]
    pub payee: Option<String>,
    /// Notes.
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
    /// Tag IDs to associate.
    #[serde(default)]
    pub tag_ids: Vec<Uuid>,
}

/// Data for updating an existing transaction.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct UpdateTransaction {
    /// Updated category (Some(None) clears, Some(Some(id)) sets, None = no change).
    pub category_id: Option<Option<Uuid>>,
    /// Updated type.
    pub transaction_type: Option<TransactionType>,
    /// Updated amount.
    pub amount: Option<Decimal>,
    /// Updated date.
    pub date: Option<Date>,
    /// Updated description.
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    /// Updated payee (Some(None) clears).
    pub payee: Option<Option<String>>,
    /// Updated notes (Some(None) clears).
    pub notes: Option<Option<String>>,
    /// Updated reviewed status.
    pub is_reviewed: Option<bool>,
    /// Updated tag IDs (replaces all).
    pub tag_ids: Option<Vec<Uuid>>,
}

/// Bulk update request for multiple transactions.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct BulkUpdateTransactions {
    /// Transaction IDs to update.
    #[validate(length(min = 1, max = 500))]
    pub transaction_ids: Vec<Uuid>,
    /// Set category (Some(None) clears).
    pub category_id: Option<Option<Uuid>>,
    /// Mark as reviewed.
    pub is_reviewed: Option<bool>,
    /// Tag IDs to add.
    #[serde(default)]
    pub add_tag_ids: Vec<Uuid>,
}

/// Query parameters for listing/filtering transactions.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TransactionListQuery {
    /// Filter by account.
    pub account_id: Option<Uuid>,
    /// Filter by category.
    pub category_id: Option<Uuid>,
    /// Filter by type (income/expense/transfer).
    pub transaction_type: Option<String>,
    /// Date range start (inclusive).
    pub date_from: Option<Date>,
    /// Date range end (inclusive).
    pub date_to: Option<Date>,
    /// Full-text search.
    pub q: Option<String>,
    /// Filter by reviewed status.
    pub is_reviewed: Option<bool>,
    /// Filter by tag ID.
    pub tag_id: Option<Uuid>,
    /// Filter by import ID.
    pub import_id: Option<Uuid>,
    /// Max items per page (default 50, max 100).
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Opaque cursor from previous response.
    pub cursor: Option<String>,
}

fn default_limit() -> i64 {
    50
}
