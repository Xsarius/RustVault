//! Category domain model.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Whether a category applies to income or expense transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CategoryType {
    /// Category for expense transactions.
    Expense,
    /// Category for income transactions.
    Income,
}

impl Default for CategoryType {
    fn default() -> Self {
        Self::Expense
    }
}

impl CategoryType {
    /// Convert from database string representation.
    pub fn from_db(s: &str) -> Self {
        match s {
            "income" => Self::Income,
            _ => Self::Expense,
        }
    }

    /// Convert to database string representation.
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Expense => "expense",
            Self::Income => "income",
        }
    }
}

impl std::fmt::Display for CategoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_db_str())
    }
}

/// A transaction category (supports hierarchical nesting via `parent_id`).
#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Category {
    /// Unique identifier.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Display name.
    pub name: String,
    /// Parent category ID (None = root category).
    pub parent_id: Option<Uuid>,
    /// Optional icon identifier.
    pub icon: Option<String>,
    /// Optional color hex code.
    pub color: Option<String>,
    /// Whether this category is for income or expense transactions.
    pub category_type: CategoryType,
    /// Display sort order.
    pub sort_order: i32,
    /// Extensible metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: OffsetDateTime,
}

/// Data required to create a new category.
#[derive(Debug, Serialize, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct NewCategory {
    /// Category name (1–100 chars).
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    /// Optional parent category ID.
    pub parent_id: Option<Uuid>,
    /// Optional icon.
    pub icon: Option<String>,
    /// Optional color.
    pub color: Option<String>,
    /// Category type (income or expense). Defaults to expense.
    #[serde(default)]
    pub category_type: CategoryType,
}

/// Data for updating an existing category.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct UpdateCategory {
    /// Updated name.
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    /// Updated parent ID (set to `null` to make root-level).
    pub parent_id: Option<Option<Uuid>>,
    /// Updated icon.
    pub icon: Option<String>,
    /// Updated color.
    pub color: Option<String>,
    /// Updated category type.
    pub category_type: Option<CategoryType>,
}

/// Bulk category creation request.
#[derive(Debug, Serialize, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct BulkCreateCategories {
    /// Categories to create.
    #[validate(length(min = 1, max = 100))]
    pub categories: Vec<NewCategory>,
}
