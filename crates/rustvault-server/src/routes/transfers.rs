//! Transfer routes.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::extractors::json::ValidatedJson;
use crate::response::{ApiError, ApiResponse, ErrorBody, PaginatedResponse};
use crate::state::AppState;

use rustvault_core::models::transfer::{
    LinkTransfer, NewTransfer, TransferDetectParams, TransferSuggestion,
};

/// `POST /api/transfers` — Create a transfer between two accounts.
#[utoipa::path(
    post,
    path = "/api/transfers",
    tag = "Transfers",
    security(("bearer" = [])),
    request_body = NewTransfer,
    responses(
        (status = 201, description = "Transfer created", body = inline(ApiResponse<rustvault_core::models::transfer::Transfer>)),
        (status = 400, description = "Validation error", body = ErrorBody),
    ),
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<NewTransfer>,
) -> Result<impl IntoResponse, ApiError> {
    let transfer = rustvault_core::services::transfer::create(
        &state.pool,
        auth.user_id,
        body.from_account_id,
        body.to_account_id,
        body.amount,
        body.date,
        body.description.as_deref(),
        body.method,
        body.received_amount,
    )
    .await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(transfer)))
}

/// `POST /api/transfers/link` — Link two existing transactions as a transfer pair.
#[utoipa::path(
    post,
    path = "/api/transfers/link",
    tag = "Transfers",
    security(("bearer" = [])),
    request_body = LinkTransfer,
    responses(
        (status = 201, description = "Transfer linked", body = inline(ApiResponse<rustvault_core::models::transfer::Transfer>)),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 409, description = "Already linked", body = ErrorBody),
    ),
)]
pub async fn link(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<LinkTransfer>,
) -> Result<impl IntoResponse, ApiError> {
    let transfer = rustvault_core::services::transfer::link(
        &state.pool,
        auth.user_id,
        body.debit_tx_id,
        body.credit_tx_id,
        body.method,
    )
    .await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(transfer)))
}

/// `DELETE /api/transfers/:id` — Unlink a transfer (transactions remain).
#[utoipa::path(
    delete,
    path = "/api/transfers/{id}",
    tag = "Transfers",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Transfer ID")),
    responses(
        (status = 204, description = "Transfer unlinked"),
        (status = 404, description = "Not found", body = ErrorBody),
    ),
)]
pub async fn unlink(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    rustvault_core::services::transfer::unlink(&state.pool, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/transfers/detect` — Detect potential transfer matches.
#[utoipa::path(
    post,
    path = "/api/transfers/detect",
    tag = "Transfers",
    security(("bearer" = [])),
    params(TransferDetectParams),
    responses(
        (status = 200, description = "Detected transfer suggestions", body = inline(PaginatedResponse<TransferSuggestion>)),
    ),
)]
pub async fn detect(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<TransferDetectParams>,
) -> Result<impl IntoResponse, ApiError> {
    let suggestions = rustvault_core::services::transfer::detect(
        &state.pool,
        auth.user_id,
        params.date_tolerance_days,
        params.amount_tolerance,
    )
    .await?;

    Ok(PaginatedResponse::from_vec(suggestions))
}
