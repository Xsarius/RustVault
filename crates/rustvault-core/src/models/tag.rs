//! Tag domain model.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// A user-defined tag for transaction labelling.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Tag {
    /// Unique identifier.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Tag name.
    pub name: String,
    /// Optional color hex code.
    pub color: Option<String>,
    /// Creation timestamp.
    pub created_at: OffsetDateTime,
}

/// Data required to create a new tag.
#[derive(Debug, Serialize, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct NewTag {
    /// Tag name (1–50 chars).
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    /// Optional color.
    pub color: Option<String>,
}

/// Data for updating an existing tag.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct UpdateTag {
    /// Updated name.
    #[validate(length(min = 1, max = 50))]
    pub name: Option<String>,
    /// Updated color.
    pub color: Option<String>,
}

/// Bulk tag creation request.
#[derive(Debug, Serialize, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct BulkCreateTags {
    /// Tags to create.
    #[validate(length(min = 1, max = 100))]
    pub tags: Vec<NewTag>,
}
