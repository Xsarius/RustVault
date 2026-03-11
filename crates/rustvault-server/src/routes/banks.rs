//! Bank CRUD routes.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::extractors::json::ValidatedJson;
use crate::response::{ApiError, ApiResponse, ErrorBody, PaginatedResponse};
use crate::state::AppState;

use rustvault_core::models::bank::{NewBank, UpdateBank};

/// `GET /api/banks` — List banks for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/banks",
    tag = "Banks",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "List of banks", body = inline(PaginatedResponse<rustvault_core::models::bank::Bank>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let banks = rustvault_core::services::bank::list(&state.pool, auth.user_id).await?;
    Ok(PaginatedResponse::from_vec(banks))
}

/// `POST /api/banks` — Create a new bank.
#[utoipa::path(
    post,
    path = "/api/banks",
    tag = "Banks",
    security(("bearer" = [])),
    request_body = NewBank,
    responses(
        (status = 201, description = "Bank created", body = inline(ApiResponse<rustvault_core::models::bank::Bank>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<NewBank>,
) -> Result<impl IntoResponse, ApiError> {
    let bank = rustvault_core::services::bank::create(
        &state.pool,
        auth.user_id,
        &body.name,
    )
    .await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(bank)))
}

/// `GET /api/banks/:id` — Get a single bank.
#[utoipa::path(
    get,
    path = "/api/banks/{id}",
    tag = "Banks",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Bank ID")),
    responses(
        (status = 200, description = "Bank details", body = inline(ApiResponse<rustvault_core::models::bank::Bank>)),
        (status = 404, description = "Bank not found", body = ErrorBody),
    ),
)]
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let bank = rustvault_core::services::bank::get(&state.pool, auth.user_id, id).await?;
    Ok(ApiResponse::ok(bank))
}

/// `PUT /api/banks/:id` — Update a bank.
#[utoipa::path(
    put,
    path = "/api/banks/{id}",
    tag = "Banks",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Bank ID")),
    request_body = UpdateBank,
    responses(
        (status = 200, description = "Bank updated", body = inline(ApiResponse<rustvault_core::models::bank::Bank>)),
        (status = 404, description = "Bank not found", body = ErrorBody),
    ),
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateBank>,
) -> Result<impl IntoResponse, ApiError> {
    let bank = rustvault_core::services::bank::update(
        &state.pool,
        auth.user_id,
        id,
        body.name.as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(bank))
}

/// `PUT /api/banks/:id/archive` — Soft-archive a bank and its accounts.
#[utoipa::path(
    put,
    path = "/api/banks/{id}/archive",
    tag = "Banks",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Bank ID")),
    responses(
        (status = 204, description = "Bank archived"),
        (status = 404, description = "Bank not found", body = ErrorBody),
    ),
)]
pub async fn archive(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    rustvault_core::services::bank::archive(&state.pool, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
