//! Account CRUD routes.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::extractors::json::ValidatedJson;
use crate::response::{ApiError, ApiResponse, ErrorBody, PaginatedResponse};
use crate::state::AppState;

use rustvault_core::models::account::{NewAccount, UpdateAccount};

/// Optional query filters for listing accounts.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct AccountListQuery {
    /// Filter by bank ID.
    pub bank_id: Option<Uuid>,
    /// Filter by account type.
    pub account_type: Option<String>,
    /// Filter by currency.
    pub currency: Option<String>,
}

/// `GET /api/accounts` — List accounts for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/accounts",
    tag = "Accounts",
    security(("bearer" = [])),
    params(AccountListQuery),
    responses(
        (status = 200, description = "List of accounts", body = inline(PaginatedResponse<rustvault_core::models::account::Account>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<AccountListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let accounts = rustvault_core::services::account::list(
        &state.pool,
        auth.user_id,
        query.bank_id,
        query.account_type.as_deref(),
        query.currency.as_deref(),
    )
    .await?;

    Ok(PaginatedResponse::from_vec(accounts))
}

/// `POST /api/accounts` — Create a new account.
#[utoipa::path(
    post,
    path = "/api/accounts",
    tag = "Accounts",
    security(("bearer" = [])),
    request_body = NewAccount,
    responses(
        (status = 201, description = "Account created", body = inline(ApiResponse<rustvault_core::models::account::Account>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<NewAccount>,
) -> Result<impl IntoResponse, ApiError> {
    let account = rustvault_core::services::account::create(
        &state.pool,
        auth.user_id,
        body.bank_id,
        &body.name,
        &body.currency,
        body.account_type,
        body.supports_nonstandard_topup,
    )
    .await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(account)))
}

/// `GET /api/accounts/:id` — Get a single account.
#[utoipa::path(
    get,
    path = "/api/accounts/{id}",
    tag = "Accounts",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Account ID")),
    responses(
        (status = 200, description = "Account details", body = inline(ApiResponse<rustvault_core::models::account::Account>)),
        (status = 404, description = "Account not found", body = ErrorBody),
    ),
)]
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let account =
        rustvault_core::services::account::get(&state.pool, auth.user_id, id).await?;
    Ok(ApiResponse::ok(account))
}

/// `PUT /api/accounts/:id` — Update an account.
#[utoipa::path(
    put,
    path = "/api/accounts/{id}",
    tag = "Accounts",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Account ID")),
    request_body = UpdateAccount,
    responses(
        (status = 200, description = "Account updated", body = inline(ApiResponse<rustvault_core::models::account::Account>)),
        (status = 404, description = "Account not found", body = ErrorBody),
    ),
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateAccount>,
) -> Result<impl IntoResponse, ApiError> {
    let account = rustvault_core::services::account::update(
        &state.pool,
        auth.user_id,
        id,
        body.name.as_deref(),
        body.currency.as_deref(),
        body.account_type,
        body.supports_nonstandard_topup,
    )
    .await?;
    Ok(ApiResponse::ok(account))
}

/// `PUT /api/accounts/:id/archive` — Soft-archive an account.
#[utoipa::path(
    put,
    path = "/api/accounts/{id}/archive",
    tag = "Accounts",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Account ID")),
    responses(
        (status = 204, description = "Account archived"),
        (status = 404, description = "Account not found", body = ErrorBody),
    ),
)]
pub async fn archive(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    rustvault_core::services::account::archive(&state.pool, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
