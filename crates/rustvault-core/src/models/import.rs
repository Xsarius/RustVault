//! Import domain model.

use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

/// Status of an import operation.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "import_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ImportStatus {
    /// File uploaded, awaiting configuration/execution.
    Pending,
    /// Import in progress.
    Processing,
    /// Import completed successfully.
    Completed,
    /// Import failed.
    Failed,
    /// Import was rolled back.
    RolledBack,
}

/// A single parsed row ready for the import pipeline.
///
/// This mirrors [`RawTransaction`] from `rustvault-import` but lives in core
/// so that the pipeline can be invoked without a circular dependency.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ParsedRow {
    /// Transaction date.
    #[schema(value_type = String, format = Date)]
    pub date: Date,
    /// Signed amount (positive = credit, negative = debit).
    pub amount: Decimal,
    /// ISO 4217 currency override (if present in file, otherwise account default).
    pub currency: Option<String>,
    /// Primary description / narrative.
    pub description: String,
    /// Payee / merchant name.
    pub payee: Option<String>,
    /// Bank reference / check number.
    pub reference: Option<String>,
    /// Extra key-value data from the parser.
    #[schema(value_type = Object)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Summary returned after executing an import.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ImportExecutionResult {
    /// The import record.
    pub import: Import,
    /// Number of transactions inserted.
    pub imported_count: i32,
    /// Number of duplicates detected and skipped.
    pub duplicate_count: i32,
    /// Number of rows that had errors.
    pub error_count: i32,
    /// Per-row error details (row index → message).
    pub errors: Vec<ImportRowError>,
    /// Rules that were auto-applied (rule ID → match count).
    pub rules_applied: HashMap<Uuid, i32>,
}

/// An error for a specific row during import.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ImportRowError {
    /// Zero-based row index in the parsed data.
    pub row: usize,
    /// Error message.
    pub message: String,
}

/// An import session tracking file ingestion.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct Import {
    /// Import ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Original file name.
    pub file_name: String,
    /// Detected file format.
    pub file_format: String,
    /// Target account ID.
    pub account_id: Uuid,
    /// Current status.
    pub status: ImportStatus,
    /// Total rows in file.
    pub total_rows: i32,
    /// Successfully imported count.
    pub imported_count: i32,
    /// Skipped row count.
    pub skipped_count: i32,
    /// Duplicate transactions detected.
    pub duplicate_count: i32,
    /// Rows with parse errors.
    pub error_count: i32,
    /// Detailed error info (JSONB).
    pub error_details: Option<serde_json::Value>,
    /// Saved column mapping.
    pub column_mapping: Option<serde_json::Value>,
    /// Metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: OffsetDateTime,
    /// Last update timestamp.
    pub updated_at: OffsetDateTime,
}
