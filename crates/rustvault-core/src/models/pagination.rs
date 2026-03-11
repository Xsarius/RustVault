//! Pagination types.

use serde::{Deserialize, Serialize};

/// Query parameters for cursor-based pagination.
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    /// Maximum items to return (default 50, max 100).
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Opaque cursor from a previous response.
    pub cursor: Option<String>,
}

fn default_limit() -> i64 {
    50
}

impl PaginationParams {
    /// Clamp the limit to the allowed range [1, 100].
    pub fn effective_limit(&self) -> i64 {
        self.limit.clamp(1, 100)
    }
}

/// Metadata for paginated responses.
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    /// Total number of matching items.
    pub total: i64,
    /// Number of items in this page.
    pub page_size: i64,
    /// Cursor for the next page (None if no more items).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Whether there are more items.
    pub has_more: bool,
}
