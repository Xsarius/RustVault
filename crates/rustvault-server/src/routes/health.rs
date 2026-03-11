//! Health-check route.

use axum::extract::State;
use axum::response::IntoResponse;

use crate::response::{ApiError, ApiResponse, ErrorBody};
use crate::state::AppState;

/// `GET /api/health` — Simple liveness probe.
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = inline(ApiResponse<serde_json::Value>)),
        (status = 500, description = "Database unreachable", body = ErrorBody),
    ),
)]
pub async fn health(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    // Verify database connectivity.
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Health-check DB query failed");
            ApiError::Internal("database unreachable".into())
        })?;

    Ok(ApiResponse::ok(serde_json::json!({ "status": "healthy" })))
}
