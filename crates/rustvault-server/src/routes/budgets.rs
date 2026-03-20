//! Budget CRUD routes — budgets, budget lines, summary, copy, and exchange rates.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::extractors::json::ValidatedJson;
use crate::response::{ApiError, ApiResponse, ErrorBody, PaginatedResponse};
use crate::state::AppState;

use rustvault_core::models::budget::{
    BudgetSummary, BulkBudgetLines, NewBudget, NewBudgetLine, UpdateBudget, UpdateBudgetLine,
};

// ── Query params ──────────────────────────────────────────────────────────────

/// Optional filters for listing budgets.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct BudgetListQuery {
    /// Include archived budgets.
    #[serde(default)]
    pub include_archived: bool,
}

/// Body for the copy-budget action.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CopyBudgetRequest {
    /// Name for the new (copied) budget.
    pub name: String,
    /// New period start.
    pub period_start: time::Date,
    /// New period end.
    pub period_end: time::Date,
}

// ── Budget handlers ───────────────────────────────────────────────────────────

/// `GET /api/budgets` — List all budgets for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/budgets",
    tag = "Budgets",
    security(("bearer" = [])),
    params(BudgetListQuery),
    responses(
        (status = 200, description = "List of budgets",
         body = inline(PaginatedResponse<rustvault_core::models::budget::Budget>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<BudgetListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let budgets =
        rustvault_core::services::budget::list(&state.pool, auth.user_id, query.include_archived)
            .await?;

    Ok(PaginatedResponse::from_vec(budgets))
}

/// `POST /api/budgets` — Create a new budget.
#[utoipa::path(
    post,
    path = "/api/budgets",
    tag = "Budgets",
    security(("bearer" = [])),
    request_body = NewBudget,
    responses(
        (status = 201, description = "Budget created",
         body = inline(ApiResponse<rustvault_core::models::budget::Budget>)),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<NewBudget>,
) -> Result<impl IntoResponse, ApiError> {
    let budget = rustvault_core::services::budget::create(&state.pool, auth.user_id, body).await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(budget)))
}

/// `GET /api/budgets/:id` — Get a single budget.
#[utoipa::path(
    get,
    path = "/api/budgets/{id}",
    tag = "Budgets",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Budget ID")),
    responses(
        (status = 200, description = "Budget details",
         body = inline(ApiResponse<rustvault_core::models::budget::Budget>)),
        (status = 404, description = "Budget not found", body = ErrorBody),
    ),
)]
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let budget = rustvault_core::services::budget::get(&state.pool, auth.user_id, id).await?;
    Ok(ApiResponse::ok(budget))
}

/// `PUT /api/budgets/:id` — Update a budget.
#[utoipa::path(
    put,
    path = "/api/budgets/{id}",
    tag = "Budgets",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Budget ID")),
    request_body = UpdateBudget,
    responses(
        (status = 200, description = "Budget updated",
         body = inline(ApiResponse<rustvault_core::models::budget::Budget>)),
        (status = 404, description = "Budget not found", body = ErrorBody),
    ),
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateBudget>,
) -> Result<impl IntoResponse, ApiError> {
    let budget =
        rustvault_core::services::budget::update(&state.pool, auth.user_id, id, body).await?;
    Ok(ApiResponse::ok(budget))
}

/// `DELETE /api/budgets/:id` — Delete a budget (and all its lines).
#[utoipa::path(
    delete,
    path = "/api/budgets/{id}",
    tag = "Budgets",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Budget ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Budget not found", body = ErrorBody),
    ),
)]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    rustvault_core::services::budget::delete(&state.pool, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/budgets/:id/summary` — Planned vs. actual summary for a budget.
///
/// Refreshes `actual_amount_cache` on all lines before returning.
#[utoipa::path(
    get,
    path = "/api/budgets/{id}/summary",
    tag = "Budgets",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Budget ID")),
    responses(
        (status = 200, description = "Budget summary", body = inline(ApiResponse<BudgetSummary>)),
        (status = 404, description = "Budget not found", body = ErrorBody),
    ),
)]
pub async fn summary(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let s = rustvault_core::services::budget::summary(&state.pool, auth.user_id, id).await?;
    Ok(ApiResponse::ok(s))
}

/// `POST /api/budgets/:id/copy` — Copy budget lines into a new period.
#[utoipa::path(
    post,
    path = "/api/budgets/{id}/copy",
    tag = "Budgets",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Source budget ID")),
    request_body = CopyBudgetRequest,
    responses(
        (status = 201, description = "New budget created",
         body = inline(ApiResponse<rustvault_core::models::budget::Budget>)),
        (status = 404, description = "Source budget not found", body = ErrorBody),
    ),
)]
pub async fn copy(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    axum::Json(body): axum::Json<CopyBudgetRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let new_budget = rustvault_core::services::budget::copy(
        &state.pool,
        auth.user_id,
        id,
        body.name,
        body.period_start,
        body.period_end,
    )
    .await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(new_budget)))
}

// ── Budget Line handlers ──────────────────────────────────────────────────────

/// `POST /api/budgets/:id/lines` — Add a line to a budget.
#[utoipa::path(
    post,
    path = "/api/budgets/{id}/lines",
    tag = "Budgets",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Budget ID")),
    request_body = NewBudgetLine,
    responses(
        (status = 201, description = "Line added",
         body = inline(ApiResponse<rustvault_core::models::budget::BudgetLine>)),
        (status = 409, description = "Category already has a line", body = ErrorBody),
    ),
)]
pub async fn add_line(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<NewBudgetLine>,
) -> Result<impl IntoResponse, ApiError> {
    let line =
        rustvault_core::services::budget::add_line(&state.pool, auth.user_id, id, body).await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(line)))
}

/// `POST /api/budgets/:id/lines/bulk` — Replace all lines for a budget.
#[utoipa::path(
    post,
    path = "/api/budgets/{id}/lines/bulk",
    tag = "Budgets",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Budget ID")),
    request_body = BulkBudgetLines,
    responses(
        (status = 200, description = "All lines replaced",
         body = inline(PaginatedResponse<rustvault_core::models::budget::BudgetLine>)),
        (status = 404, description = "Budget not found", body = ErrorBody),
    ),
)]
pub async fn bulk_set_lines(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    axum::Json(body): axum::Json<BulkBudgetLines>,
) -> Result<impl IntoResponse, ApiError> {
    let lines =
        rustvault_core::services::budget::bulk_set_lines(&state.pool, auth.user_id, id, body.lines)
            .await?;
    Ok(PaginatedResponse::from_vec(lines))
}

/// `PUT /api/budgets/:id/lines/:line_id` — Update a budget line.
#[utoipa::path(
    put,
    path = "/api/budgets/{id}/lines/{line_id}",
    tag = "Budgets",
    security(("bearer" = [])),
    params(
        ("id" = Uuid, Path, description = "Budget ID"),
        ("line_id" = Uuid, Path, description = "Line ID"),
    ),
    request_body = UpdateBudgetLine,
    responses(
        (status = 200, description = "Line updated",
         body = inline(ApiResponse<rustvault_core::models::budget::BudgetLine>)),
        (status = 404, description = "Not found", body = ErrorBody),
    ),
)]
pub async fn update_line(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, line_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(body): ValidatedJson<UpdateBudgetLine>,
) -> Result<impl IntoResponse, ApiError> {
    let line =
        rustvault_core::services::budget::update_line(&state.pool, auth.user_id, id, line_id, body)
            .await?;
    Ok(ApiResponse::ok(line))
}

/// `DELETE /api/budgets/:id/lines/:line_id` — Remove a budget line.
#[utoipa::path(
    delete,
    path = "/api/budgets/{id}/lines/{line_id}",
    tag = "Budgets",
    security(("bearer" = [])),
    params(
        ("id" = Uuid, Path, description = "Budget ID"),
        ("line_id" = Uuid, Path, description = "Line ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found", body = ErrorBody),
    ),
)]
pub async fn delete_line(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, line_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    rustvault_core::services::budget::delete_line(&state.pool, auth.user_id, id, line_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/budgets/:id/generate-next` — Generate the next period for a recurring budget (P4.6).
#[utoipa::path(
    post,
    path = "/api/budgets/{id}/generate-next",
    tag = "Budgets",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Source recurring budget ID")),
    responses(
        (status = 201, description = "Next-period budget created",
         body = inline(ApiResponse<rustvault_core::models::budget::Budget>)),
        (status = 400, description = "Budget is not recurring or unsupported rule", body = ErrorBody),
        (status = 404, description = "Budget not found", body = ErrorBody),
    ),
)]
pub async fn generate_next(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let budget =
        rustvault_core::services::budget::generate_next_period(&state.pool, auth.user_id, id)
            .await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(budget)))
}

// (Exchange rates are refreshed by a scheduled background task; no REST endpoints needed.)
