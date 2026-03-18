//! Import routes — upload, preview, configure, execute, list, detail, rollback.

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::extractors::json::ValidatedJson;
use crate::response::{ApiError, ApiResponse, ErrorBody, PaginatedResponse};
use crate::state::AppState;

use rustvault_core::models::import::{Import, ImportExecutionResult, ParsedRow};
use rustvault_import::{ColumnMapping, ParserRegistry, RawTransaction};

// ── helpers ────────────────────────────────────────────────────

/// Convert a [`RawTransaction`] (from `rustvault-import`) into a [`ParsedRow`]
/// (from `rustvault-core`) without introducing a cross-crate dependency.
fn raw_to_parsed(raw: RawTransaction) -> ParsedRow {
    ParsedRow {
        date: raw.date,
        amount: raw.amount,
        currency: raw.currency,
        description: raw.description,
        payee: raw.payee,
        reference: raw.reference,
        metadata: raw.metadata,
    }
}

/// Parse a human-readable file-size string (e.g. `"50MB"`) into bytes.
fn parse_max_file_size(s: &str) -> usize {
    let s = s.trim().to_uppercase();
    if let Some(num) = s.strip_suffix("GB") {
        num.trim().parse::<usize>().unwrap_or(50) * 1024 * 1024 * 1024
    } else if let Some(num) = s.strip_suffix("MB") {
        num.trim().parse::<usize>().unwrap_or(50) * 1024 * 1024
    } else if let Some(num) = s.strip_suffix("KB") {
        num.trim().parse::<usize>().unwrap_or(50) * 1024
    } else {
        // Default to 50 MB.
        50 * 1024 * 1024
    }
}

/// Extract file extension from a filename (lowercase, no dot).
fn file_extension(name: &str) -> Option<String> {
    name.rsplit('.').next().map(|ext| ext.to_lowercase())
}

// ── request / response bodies ──────────────────────────────────

/// Request body for the configure (save mapping) endpoint.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct ConfigureImportRequest {
    /// Column mapping configuration.
    pub mapping: serde_json::Value,
}

/// Response body for the upload endpoint.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UploadResponse {
    /// The created import record.
    pub import: Import,
    /// Detected file format.
    pub detected_format: String,
    /// Preview of the first rows (up to 10).
    pub preview: Vec<ParsedRow>,
    /// Total rows detected.
    pub total_rows: usize,
}

// ── route handlers ─────────────────────────────────────────────

/// `POST /api/imports/upload` — Upload a bank statement file.
///
/// Expects a `multipart/form-data` request with:
/// - `file` — the bank statement file
/// - `account_id` — UUID of the target account
/// - `mapping` (optional) — JSON column mapping
#[utoipa::path(
    post,
    path = "/api/imports/upload",
    tag = "Imports",
    security(("bearer" = [])),
    request_body(content_type = "multipart/form-data", content = inline(String), description = "Form fields: file (binary), account_id (UUID), mapping (optional JSON)"),
    responses(
        (status = 201, description = "File uploaded and parsed", body = inline(ApiResponse<UploadResponse>)),
        (status = 400, description = "Bad request", body = ErrorBody),
        (status = 404, description = "Account not found", body = ErrorBody),
    ),
)]
pub async fn upload(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let max_size = parse_max_file_size(&state.config.import.max_file_size);

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut account_id: Option<Uuid> = None;
    let mut mapping: Option<ColumnMapping> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("multipart error: {e}")))?
    {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(String::from);
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("failed to read file: {e}")))?;
                if bytes.len() > max_size {
                    return Err(ApiError::BadRequest(format!(
                        "file too large (max {})",
                        state.config.import.max_file_size
                    )));
                }
                file_bytes = Some(bytes.to_vec());
            }
            "account_id" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("invalid account_id: {e}")))?;
                account_id =
                    Some(Uuid::parse_str(&text).map_err(|_| {
                        ApiError::BadRequest("account_id must be a valid UUID".into())
                    })?);
            }
            "mapping" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("invalid mapping: {e}")))?;
                mapping = Some(
                    serde_json::from_str(&text)
                        .map_err(|e| ApiError::BadRequest(format!("invalid mapping JSON: {e}")))?,
                );
            }
            _ => {
                // Ignore unknown fields.
            }
        }
    }

    let file_bytes =
        file_bytes.ok_or_else(|| ApiError::BadRequest("missing required field: file".into()))?;
    let file_name =
        file_name.ok_or_else(|| ApiError::BadRequest("file must have a filename".into()))?;
    let account_id = account_id
        .ok_or_else(|| ApiError::BadRequest("missing required field: account_id".into()))?;

    // Validate extension.
    let ext = file_extension(&file_name)
        .ok_or_else(|| ApiError::BadRequest("file must have an extension".into()))?;
    if !state.config.import.allowed_extensions.contains(&ext) {
        return Err(ApiError::BadRequest(format!(
            "unsupported file extension: .{ext}"
        )));
    }

    // Detect format.
    let registry = ParserRegistry::new();
    let (parser, format) = registry
        .detect_and_select(&file_bytes, Some(&ext))
        .ok_or_else(|| ApiError::BadRequest("could not detect file format".into()))?;

    let format_name = format!("{format:?}").to_lowercase();

    // Parse a preview (first 10 rows).
    let mapping_ref = mapping.as_ref();
    let preview_raw = parser
        .preview(&file_bytes, mapping_ref, 10)
        .map_err(|e| ApiError::BadRequest(format!("parse error: {e}")))?;

    // Parse all rows to count total.
    let all_raw = parser
        .parse(&file_bytes, mapping_ref)
        .map_err(|e| ApiError::BadRequest(format!("parse error: {e}")))?;
    let total_rows = all_raw.len();

    // Create the import record.
    let import = rustvault_core::services::import::create(
        &state.pool,
        auth.user_id,
        &file_name,
        &format_name,
        account_id,
    )
    .await?;

    // Save mapping if provided.
    if let Some(ref m) = mapping {
        let mapping_json = serde_json::to_value(m)
            .map_err(|e| ApiError::Internal(format!("mapping serialization: {e}")))?;
        rustvault_core::services::import::save_mapping(
            &state.pool,
            auth.user_id,
            import.id,
            &mapping_json,
        )
        .await?;
    }

    let preview: Vec<ParsedRow> = preview_raw.into_iter().map(raw_to_parsed).collect();

    Ok((
        StatusCode::CREATED,
        ApiResponse::ok(UploadResponse {
            import,
            detected_format: format_name,
            preview,
            total_rows,
        }),
    ))
}

/// `POST /api/imports/:id/preview` — Re-preview an import with updated column mapping.
#[utoipa::path(
    post,
    path = "/api/imports/{id}/preview",
    tag = "Imports",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Import ID")),
    request_body = ConfigureImportRequest,
    responses(
        (status = 200, description = "Preview with updated mapping", body = inline(ApiResponse<Vec<ParsedRow>>)),
        (status = 400, description = "Parse error", body = ErrorBody),
        (status = 404, description = "Import not found", body = ErrorBody),
    ),
)]
pub async fn preview(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(_body): ValidatedJson<ConfigureImportRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify import exists.
    let _import = rustvault_core::services::import::get(&state.pool, auth.user_id, id).await?;

    // The preview endpoint currently requires the raw file data to be
    // re-sent. In a production system you'd store the file temporarily
    // (e.g. S3 / temp dir). For now, we return a helpful error.
    Err::<axum::Json<()>, _>(ApiError::BadRequest(
        "preview with re-mapping requires re-uploading the file via the upload endpoint with the updated mapping".into(),
    ))
}

/// `PUT /api/imports/:id/configure` — Save column mapping configuration.
#[utoipa::path(
    put,
    path = "/api/imports/{id}/configure",
    tag = "Imports",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Import ID")),
    request_body = ConfigureImportRequest,
    responses(
        (status = 200, description = "Mapping saved", body = inline(ApiResponse<Import>)),
        (status = 404, description = "Import not found", body = ErrorBody),
    ),
)]
pub async fn configure(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<ConfigureImportRequest>,
) -> Result<impl IntoResponse, ApiError> {
    rustvault_core::services::import::save_mapping(&state.pool, auth.user_id, id, &body.mapping)
        .await?;
    let import = rustvault_core::services::import::get(&state.pool, auth.user_id, id).await?;
    Ok(ApiResponse::ok(import))
}

/// `POST /api/imports/:id/execute` — Re-upload the file and run the full import.
///
/// Accepts the same bank statement file that was uploaded, re-parses it using
/// the mapping saved via the configure endpoint (or an optional mapping
/// override), then runs:
///
/// ```text
/// Parse → Deduplicate → Auto-Categorise → Detect Transfers → Persist → Summary
/// ```
///
/// Multipart fields:
/// - `file` — the bank statement file (binary).
/// - `mapping` (optional) — JSON column mapping override; overrides any
///   previously saved mapping for this import.
/// - `skip_duplicates` (optional, default `"true"`) — `"true"` / `"1"` to
///   skip transactions that look like duplicates.
#[utoipa::path(
    post,
    path = "/api/imports/{id}/execute",
    tag = "Imports",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Import ID")),
    request_body(content_type = "multipart/form-data", content = inline(String), description = "Form fields: file (binary), mapping (optional JSON override), skip_duplicates (optional bool)"),
    responses(
        (status = 200, description = "Import executed", body = inline(ApiResponse<ImportExecutionResult>)),
        (status = 400, description = "Bad request / file parse error", body = ErrorBody),
        (status = 404, description = "Import not found", body = ErrorBody),
    ),
)]
pub async fn execute(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    // Look up the existing import record.
    let import = rustvault_core::services::import::get(&state.pool, auth.user_id, id).await?;

    if import.status != rustvault_core::models::import::ImportStatus::Pending {
        return Err(ApiError::BadRequest(format!(
            "import is already {:?} — only pending imports can be executed",
            import.status
        )));
    }

    let max_size = parse_max_file_size(&state.config.import.max_file_size);
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut mapping_override: Option<ColumnMapping> = None;
    let mut skip_duplicates = true;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("multipart error: {e}")))?
    {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(String::from);
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("failed to read file: {e}")))?;
                if bytes.len() > max_size {
                    return Err(ApiError::BadRequest(format!(
                        "file too large (max {})",
                        state.config.import.max_file_size
                    )));
                }
                file_bytes = Some(bytes.to_vec());
            }
            "mapping" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("invalid mapping: {e}")))?;
                mapping_override = Some(
                    serde_json::from_str(&text)
                        .map_err(|e| ApiError::BadRequest(format!("invalid mapping JSON: {e}")))?,
                );
            }
            "skip_duplicates" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("invalid skip_duplicates: {e}")))?;
                skip_duplicates = text.trim().eq_ignore_ascii_case("true") || text.trim() == "1";
            }
            _ => {}
        }
    }

    let file_bytes =
        file_bytes.ok_or_else(|| ApiError::BadRequest("missing required field: file".into()))?;
    let file_name =
        file_name.ok_or_else(|| ApiError::BadRequest("file must have a filename".into()))?;

    // Validate extension.
    let ext = file_extension(&file_name)
        .ok_or_else(|| ApiError::BadRequest("file must have an extension".into()))?;
    if !state.config.import.allowed_extensions.contains(&ext) {
        return Err(ApiError::BadRequest(format!(
            "unsupported file extension: .{ext}"
        )));
    }

    // Save mapping override before resolving the effective mapping.
    if let Some(ref m) = mapping_override {
        let mapping_json = serde_json::to_value(m)
            .map_err(|e| ApiError::Internal(format!("mapping serialization: {e}")))?;
        rustvault_core::services::import::save_mapping(
            &state.pool,
            auth.user_id,
            import.id,
            &mapping_json,
        )
        .await?;
    }

    // Resolve effective mapping: runtime override first, then import's saved mapping.
    let effective_mapping: Option<ColumnMapping> = mapping_override.or_else(|| {
        import
            .column_mapping
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    });

    // Detect format and parse.
    let registry = ParserRegistry::new();
    let (parser, _format) = registry
        .detect_and_select(&file_bytes, Some(&ext))
        .ok_or_else(|| ApiError::BadRequest("could not detect file format".into()))?;

    let all_raw = parser
        .parse(&file_bytes, effective_mapping.as_ref())
        .map_err(|e| ApiError::BadRequest(format!("parse error: {e}")))?;

    let parsed_rows: Vec<ParsedRow> = all_raw.into_iter().map(raw_to_parsed).collect();

    let result = rustvault_core::services::import::execute(
        &state.pool,
        auth.user_id,
        import.id,
        import.account_id,
        &parsed_rows,
        skip_duplicates,
    )
    .await?;

    Ok(ApiResponse::ok(result))
}

/// `POST /api/imports/upload-and-execute` — Upload and immediately execute an import.
///
/// Convenience endpoint that combines upload + execute in a single request.
/// Accepts the same multipart fields as the upload endpoint plus `skip_duplicates`.
///
/// Multipart fields:
/// - `file` — the bank statement file (binary).
/// - `account_id` — UUID of the target account.
/// - `mapping` (optional) — JSON column mapping.
/// - `skip_duplicates` (optional, default `"true"`) — `"true"` / `"1"` to skip duplicates.
#[utoipa::path(
    post,
    path = "/api/imports/upload-and-execute",
    tag = "Imports",
    security(("bearer" = [])),
    request_body(content_type = "multipart/form-data", content = inline(String), description = "Form fields: file (binary), account_id (UUID), mapping (optional JSON), skip_duplicates (optional bool)"),
    responses(
        (status = 200, description = "Import executed", body = inline(ApiResponse<ImportExecutionResult>)),
        (status = 400, description = "Bad request / file parse error", body = ErrorBody),
        (status = 404, description = "Account not found", body = ErrorBody),
    ),
)]
pub async fn upload_and_execute(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let max_size = parse_max_file_size(&state.config.import.max_file_size);

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut account_id: Option<Uuid> = None;
    let mut mapping: Option<ColumnMapping> = None;
    let mut skip_duplicates = true;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("multipart error: {e}")))?
    {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(String::from);
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("failed to read file: {e}")))?;
                if bytes.len() > max_size {
                    return Err(ApiError::BadRequest(format!(
                        "file too large (max {})",
                        state.config.import.max_file_size
                    )));
                }
                file_bytes = Some(bytes.to_vec());
            }
            "account_id" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("invalid account_id: {e}")))?;
                account_id =
                    Some(Uuid::parse_str(&text).map_err(|_| {
                        ApiError::BadRequest("account_id must be a valid UUID".into())
                    })?);
            }
            "mapping" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("invalid mapping: {e}")))?;
                mapping = Some(
                    serde_json::from_str(&text)
                        .map_err(|e| ApiError::BadRequest(format!("invalid mapping JSON: {e}")))?,
                );
            }
            "skip_duplicates" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("invalid skip_duplicates: {e}")))?;
                skip_duplicates = text.trim().eq_ignore_ascii_case("true") || text.trim() == "1";
            }
            _ => {}
        }
    }

    let file_bytes =
        file_bytes.ok_or_else(|| ApiError::BadRequest("missing required field: file".into()))?;
    let file_name =
        file_name.ok_or_else(|| ApiError::BadRequest("file must have a filename".into()))?;
    let account_id = account_id
        .ok_or_else(|| ApiError::BadRequest("missing required field: account_id".into()))?;

    // Validate extension.
    let ext = file_extension(&file_name)
        .ok_or_else(|| ApiError::BadRequest("file must have an extension".into()))?;
    if !state.config.import.allowed_extensions.contains(&ext) {
        return Err(ApiError::BadRequest(format!(
            "unsupported file extension: .{ext}"
        )));
    }

    // Detect format and parse.
    let registry = ParserRegistry::new();
    let (parser, format) = registry
        .detect_and_select(&file_bytes, Some(&ext))
        .ok_or_else(|| ApiError::BadRequest("could not detect file format".into()))?;

    let format_name = format!("{format:?}").to_lowercase();
    let mapping_ref = mapping.as_ref();
    let all_raw = parser
        .parse(&file_bytes, mapping_ref)
        .map_err(|e| ApiError::BadRequest(format!("parse error: {e}")))?;

    let parsed_rows: Vec<ParsedRow> = all_raw.into_iter().map(raw_to_parsed).collect();

    // Create the import record.
    let import = rustvault_core::services::import::create(
        &state.pool,
        auth.user_id,
        &file_name,
        &format_name,
        account_id,
    )
    .await?;

    // Save mapping if provided.
    if let Some(ref m) = mapping {
        let mapping_json = serde_json::to_value(m)
            .map_err(|e| ApiError::Internal(format!("mapping serialization: {e}")))?;
        rustvault_core::services::import::save_mapping(
            &state.pool,
            auth.user_id,
            import.id,
            &mapping_json,
        )
        .await?;
    }

    // Execute immediately.
    let result = rustvault_core::services::import::execute(
        &state.pool,
        auth.user_id,
        import.id,
        import.account_id,
        &parsed_rows,
        skip_duplicates,
    )
    .await?;

    Ok(ApiResponse::ok(result))
}

/// `GET /api/imports` — List past imports.
#[utoipa::path(
    get,
    path = "/api/imports",
    tag = "Imports",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "List of imports", body = inline(PaginatedResponse<Import>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let imports = rustvault_core::services::import::list(&state.pool, auth.user_id).await?;
    Ok(PaginatedResponse::from_vec(imports))
}

/// `GET /api/imports/:id` — Get import details.
#[utoipa::path(
    get,
    path = "/api/imports/{id}",
    tag = "Imports",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Import ID")),
    responses(
        (status = 200, description = "Import details", body = inline(ApiResponse<Import>)),
        (status = 404, description = "Not found", body = ErrorBody),
    ),
)]
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let import = rustvault_core::services::import::get(&state.pool, auth.user_id, id).await?;
    Ok(ApiResponse::ok(import))
}

/// `DELETE /api/imports/:id` — Rollback an import (delete all transactions from this import).
#[utoipa::path(
    delete,
    path = "/api/imports/{id}",
    tag = "Imports",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Import ID")),
    responses(
        (status = 204, description = "Import rolled back"),
        (status = 404, description = "Not found", body = ErrorBody),
    ),
)]
pub async fn rollback(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    rustvault_core::services::import::rollback(&state.pool, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
