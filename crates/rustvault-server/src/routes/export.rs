//! Export routes (P7.10) — `GET /api/export`
//!
//! Allows authenticated users to download all their transactions in
//! CSV, JSON, or QIF format.  Optional query parameters narrow the
//! export to a date range or a specific account.

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use serde::Deserialize;
use time::Date;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::response::{ApiError, ErrorBody};
use crate::state::AppState;

use rustvault_core::services::export::ExportFormat;

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ExportQuery {
    /// Export format: `csv`, `json`, or `qif`.
    pub format: String,
    /// Optional start date filter (ISO 8601, e.g. `"2026-01-01"`).
    #[param(value_type = Option<String>, format = Date)]
    pub date_from: Option<Date>,
    /// Optional end date filter (ISO 8601, e.g. `"2026-12-31"`).
    #[param(value_type = Option<String>, format = Date)]
    pub date_to: Option<Date>,
    /// Optional account UUID filter.
    pub account_id: Option<Uuid>,
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// `GET /api/export` — Export all transactions in the requested format.
///
/// Returns the file as an attachment with an appropriate `Content-Type` and
/// `Content-Disposition` header.
#[utoipa::path(
    get,
    path = "/api/export",
    tag = "Export",
    security(("bearer" = [])),
    params(ExportQuery),
    responses(
        (status = 200, description = "Transaction export file",
         content_type = "text/csv"),
        (status = 400, description = "Invalid format parameter", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn export(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let format: ExportFormat = query.format.parse().map_err(|e: String| {
        ApiError::BadRequest(format!(
            "invalid format '{fmt}': {e}. Use csv, json, or qif.",
            fmt = query.format
        ))
    })?;

    let (mime, filename, bytes) = rustvault_core::services::export::export_transactions(
        &state.pool,
        auth.user_id,
        format,
        query.date_from,
        query.date_to,
        query.account_id,
    )
    .await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        mime.parse()
            .map_err(|_| ApiError::Internal("invalid content-type".into()))?,
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{filename}\"")
            .parse()
            .map_err(|_| ApiError::Internal("invalid content-disposition".into()))?,
    );

    Ok((StatusCode::OK, headers, bytes))
}
