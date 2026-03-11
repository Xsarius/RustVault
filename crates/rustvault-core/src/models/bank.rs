//! Bank domain model.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// A banking institution configured by the user.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Bank {
    /// Unique identifier.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Display name (e.g., "Revolut", "ING").
    pub name: String,
    /// Whether the bank is soft-archived.
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

/// Data required to create a new bank.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct NewBank {
    /// Bank name (1–100 chars).
    #[validate(length(min = 1, max = 100))]
    pub name: String,
}

/// Data for updating an existing bank.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct UpdateBank {
    /// Updated name.
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
}
