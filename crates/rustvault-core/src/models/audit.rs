//! Audit log model.

use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

/// An entry in the audit log tracking entity mutations.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AuditEntry {
    /// Unique identifier.
    pub id: Uuid,
    /// User who performed the action.
    pub user_id: Option<Uuid>,
    /// Type of entity (e.g., "bank", "account", "category").
    pub entity_type: String,
    /// ID of the affected entity.
    pub entity_id: Uuid,
    /// Action performed (e.g., "create", "update", "delete").
    pub action: String,
    /// Previous state (None for creates).
    pub old_value: Option<serde_json::Value>,
    /// New state (None for deletes).
    pub new_value: Option<serde_json::Value>,
    /// When the action occurred.
    pub created_at: OffsetDateTime,
}

/// Data required to create a new audit entry.
pub struct NewAuditEntry {
    /// User who performed the action.
    pub user_id: Uuid,
    /// Type of entity.
    pub entity_type: String,
    /// ID of the affected entity.
    pub entity_id: Uuid,
    /// Action performed.
    pub action: String,
    /// Previous state.
    pub old_value: Option<serde_json::Value>,
    /// New state.
    pub new_value: Option<serde_json::Value>,
}
