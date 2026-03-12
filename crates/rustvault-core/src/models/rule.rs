//! Auto-categorization rule domain model.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// A user-defined auto-categorization rule.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AutoRule {
    /// Rule ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Rule name.
    pub name: String,
    /// Execution priority (lower = higher priority).
    pub priority: i32,
    /// Whether the rule is active.
    pub is_enabled: bool,
    /// Conditions that must match (JSONB array).
    pub conditions: serde_json::Value,
    /// Actions to apply when matched (JSONB array).
    pub actions: serde_json::Value,
    /// Metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: OffsetDateTime,
    /// Last update timestamp.
    pub updated_at: OffsetDateTime,
}

/// Data for creating a new auto-rule.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct NewAutoRule {
    /// Rule name.
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    /// Priority (lower = higher priority).
    pub priority: Option<i32>,
    /// Conditions (JSONB array).
    pub conditions: serde_json::Value,
    /// Actions (JSONB array).
    pub actions: serde_json::Value,
}

/// Data for updating an auto-rule.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct UpdateAutoRule {
    /// Updated name.
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,
    /// Updated priority.
    pub priority: Option<i32>,
    /// Enable/disable.
    pub is_enabled: Option<bool>,
    /// Updated conditions.
    pub conditions: Option<serde_json::Value>,
    /// Updated actions.
    pub actions: Option<serde_json::Value>,
}
