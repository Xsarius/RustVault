//! Import repository — SQL operations for the `imports` table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DbError;

/// Row type matching the `imports` table schema.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ImportRow {
    /// Import ID.
    pub id: Uuid,
    /// Owner user ID.
    pub user_id: Uuid,
    /// Original file name.
    pub file_name: String,
    /// Detected file format (csv, mt940, ofx, etc.).
    pub file_format: String,
    /// Target account for imported transactions.
    pub account_id: Uuid,
    /// Current import status (as text from enum cast).
    pub status: String,
    /// Total rows in the file.
    pub total_rows: i32,
    /// Successfully imported count.
    pub imported_count: i32,
    /// Skipped (filtered out) count.
    pub skipped_count: i32,
    /// Duplicate transactions detected.
    pub duplicate_count: i32,
    /// Rows that failed to parse.
    pub error_count: i32,
    /// Detailed error information (JSONB).
    pub error_details: Option<serde_json::Value>,
    /// Saved column mapping configuration (JSONB).
    pub column_mapping: Option<serde_json::Value>,
    /// Metadata (JSONB).
    pub metadata: serde_json::Value,
    /// Creation timestamp.
    pub created_at: time::OffsetDateTime,
    /// Last update timestamp.
    pub updated_at: time::OffsetDateTime,
}

/// List imports for a user.
pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<ImportRow>, DbError> {
    let rows = sqlx::query_as::<_, ImportRow>(
        "SELECT id, user_id, file_name, file_format, account_id,
                status::text, total_rows, imported_count, skipped_count,
                duplicate_count, error_count, error_details, column_mapping,
                metadata, created_at, updated_at
         FROM imports WHERE user_id = $1
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Find an import by ID (owned by user).
pub async fn find_by_id(
    pool: &PgPool,
    user_id: Uuid,
    import_id: Uuid,
) -> Result<ImportRow, DbError> {
    sqlx::query_as::<_, ImportRow>(
        "SELECT id, user_id, file_name, file_format, account_id,
                status::text, total_rows, imported_count, skipped_count,
                duplicate_count, error_count, error_details, column_mapping,
                metadata, created_at, updated_at
         FROM imports WHERE id = $1 AND user_id = $2",
    )
    .bind(import_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Insert a new import record.
pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    file_name: &str,
    file_format: &str,
    account_id: Uuid,
) -> Result<ImportRow, DbError> {
    sqlx::query_as::<_, ImportRow>(
        "INSERT INTO imports (user_id, file_name, file_format, account_id)
         VALUES ($1, $2, $3, $4)
         RETURNING id, user_id, file_name, file_format, account_id,
                   status::text, total_rows, imported_count, skipped_count,
                   duplicate_count, error_count, error_details, column_mapping,
                   metadata, created_at, updated_at",
    )
    .bind(user_id)
    .bind(file_name)
    .bind(file_format)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            DbError::ForeignKeyViolation("account_id".into())
        }
        _ => DbError::Sqlx(e),
    })
}

/// Update import status and counts.
#[expect(clippy::too_many_arguments)]
pub async fn update_status(
    pool: &PgPool,
    user_id: Uuid,
    import_id: Uuid,
    status: &str,
    total_rows: i32,
    imported_count: i32,
    skipped_count: i32,
    duplicate_count: i32,
    error_count: i32,
    error_details: Option<&serde_json::Value>,
) -> Result<ImportRow, DbError> {
    sqlx::query_as::<_, ImportRow>(
        "UPDATE imports
         SET status = $3::import_status,
             total_rows = $4,
             imported_count = $5,
             skipped_count = $6,
             duplicate_count = $7,
             error_count = $8,
             error_details = $9
         WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, file_name, file_format, account_id,
                   status::text, total_rows, imported_count, skipped_count,
                   duplicate_count, error_count, error_details, column_mapping,
                   metadata, created_at, updated_at",
    )
    .bind(import_id)
    .bind(user_id)
    .bind(status)
    .bind(total_rows)
    .bind(imported_count)
    .bind(skipped_count)
    .bind(duplicate_count)
    .bind(error_count)
    .bind(error_details)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)
}

/// Save column mapping for an import.
pub async fn save_column_mapping(
    pool: &PgPool,
    user_id: Uuid,
    import_id: Uuid,
    column_mapping: &serde_json::Value,
) -> Result<(), DbError> {
    let result = sqlx::query(
        "UPDATE imports SET column_mapping = $3 WHERE id = $1 AND user_id = $2",
    )
    .bind(import_id)
    .bind(user_id)
    .bind(column_mapping)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}
