//! Transfer domain model.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Method used for the transfer.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "transfer_method", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TransferMethod {
    /// Internal bank transfer.
    Internal,
    /// Card payment (e.g. Revolut top-up).
    CardPayment,
    /// Wire transfer.
    Wire,
    /// Other method.
    Other,
}

/// Status of a transfer link.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "transfer_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TransferStatus {
    /// Suggested by auto-detection.
    Suggested,
    /// Confirmed by user.
    Confirmed,
    /// Rejected by user.
    Rejected,
}

/// A transfer linking two transactions (debit + credit).
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct Transfer {
    /// Transfer ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Debit (outgoing) transaction ID.
    pub debit_tx_id: Uuid,
    /// Credit (incoming) transaction ID.
    pub credit_tx_id: Uuid,
    /// Transfer method.
    pub method: TransferMethod,
    /// Transfer status.
    pub status: TransferStatus,
    /// Exchange rate (if cross-currency).
    pub exchange_rate: Option<Decimal>,
    /// Match confidence (0–100).
    pub confidence: Option<Decimal>,
    /// Notes.
    pub notes: Option<String>,
    /// Metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: OffsetDateTime,
}

/// Data for creating a new transfer (auto-creates debit + credit transactions).
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct NewTransfer {
    /// Source (debit) account ID.
    pub from_account_id: Uuid,
    /// Destination (credit) account ID.
    pub to_account_id: Uuid,
    /// Amount to transfer.
    pub amount: Decimal,
    /// Transfer date.
    pub date: time::Date,
    /// Description.
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    /// Transfer method.
    #[serde(default = "default_method")]
    pub method: TransferMethod,
    /// Amount received (for cross-currency, defaults to `amount`).
    pub received_amount: Option<Decimal>,
}

fn default_method() -> TransferMethod {
    TransferMethod::Internal
}

/// Data for linking two existing transactions as a transfer.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct LinkTransfer {
    /// Debit transaction ID (outgoing).
    pub debit_tx_id: Uuid,
    /// Credit transaction ID (incoming).
    pub credit_tx_id: Uuid,
    /// Transfer method.
    #[serde(default = "default_method")]
    pub method: TransferMethod,
}

/// Parameters for transfer detection.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TransferDetectParams {
    /// Date tolerance in days (default: 3).
    #[serde(default = "default_date_tolerance")]
    pub date_tolerance_days: i32,
    /// Amount tolerance (default: 0.00 — exact match).
    #[serde(default)]
    pub amount_tolerance: Decimal,
}

fn default_date_tolerance() -> i32 {
    3
}

/// A suggested transfer match.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct TransferSuggestion {
    /// Debit transaction ID.
    pub debit_tx_id: Uuid,
    /// Credit transaction ID.
    pub credit_tx_id: Uuid,
    /// Debit account ID.
    pub debit_account_id: Uuid,
    /// Credit account ID.
    pub credit_account_id: Uuid,
    /// Debit amount.
    pub debit_amount: Decimal,
    /// Credit amount.
    pub credit_amount: Decimal,
    /// Debit description.
    pub debit_desc: String,
    /// Credit description.
    pub credit_desc: String,
    /// Match confidence (0–100).
    pub confidence: Decimal,
}
