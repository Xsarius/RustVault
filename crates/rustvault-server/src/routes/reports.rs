//! Report routes — visualisation and analysis endpoints (Phase 5).

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;
use time::Date;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::response::{ApiError, ApiResponse, ErrorBody};
use crate::state::AppState;

// ── Query params ──────────────────────────────────────────────────────────────

/// Date range used by most report endpoints.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct DateRangeQuery {
    /// Range start (ISO 8601 date, e.g. `"2024-01-01"`).
    #[param(value_type = String, format = Date)]
    pub from: Date,
    /// Range end (ISO 8601 date, e.g. `"2024-12-31"`).
    #[param(value_type = String, format = Date)]
    pub to: Date,
}

/// Query params for balance history — optional account filter.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct BalanceHistoryQuery {
    /// Range start (ISO 8601 date).
    #[param(value_type = String, format = Date)]
    pub from: Date,
    /// Range end (ISO 8601 date).
    #[param(value_type = String, format = Date)]
    pub to: Date,
    /// Comma-separated list of account UUIDs to include.
    /// Omit to include all non-archived accounts.
    pub account_ids: Option<String>,
}

impl BalanceHistoryQuery {
    /// Parse the comma-separated `account_ids` string into a `Vec<Uuid>`.
    pub fn parsed_account_ids(&self) -> Vec<Uuid> {
        self.account_ids
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.trim().parse::<Uuid>().ok())
            .collect()
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// `GET /api/reports/summary` — Dashboard summary: net worth, monthly totals, trends.
#[utoipa::path(
    get,
    path = "/api/reports/summary",
    tag = "Reports",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "Dashboard summary",
         body = inline(ApiResponse<rustvault_core::models::report::DashboardSummary>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn summary(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let data = rustvault_core::services::report::summary(&state.pool, auth.user_id).await?;
    Ok(ApiResponse::ok(data))
}

/// `GET /api/reports/income-expense` — Monthly income vs. expense with category breakdown.
#[utoipa::path(
    get,
    path = "/api/reports/income-expense",
    tag = "Reports",
    security(("bearer" = [])),
    params(DateRangeQuery),
    responses(
        (status = 200, description = "Income vs expense report",
         body = inline(ApiResponse<rustvault_core::models::report::IncomeExpenseReport>)),
        (status = 400, description = "Invalid query params", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn income_expense(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<DateRangeQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let data =
        rustvault_core::services::report::income_expense(&state.pool, auth.user_id, q.from, q.to)
            .await?;
    Ok(ApiResponse::ok(data))
}

/// `GET /api/reports/categories/:id/trend` — Monthly spending trend for a single category.
#[utoipa::path(
    get,
    path = "/api/reports/categories/{id}/trend",
    tag = "Reports",
    security(("bearer" = [])),
    params(
        ("id" = Uuid, Path, description = "Category ID"),
        DateRangeQuery,
    ),
    responses(
        (status = 200, description = "Category trend",
         body = inline(ApiResponse<rustvault_core::models::report::CategoryTrendReport>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 404, description = "Category not found", body = ErrorBody),
    ),
)]
pub async fn category_trend(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<DateRangeQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let data = rustvault_core::services::report::category_trend(
        &state.pool,
        auth.user_id,
        id,
        q.from,
        q.to,
    )
    .await?;
    Ok(ApiResponse::ok(data))
}

/// `GET /api/reports/balance-history` — Historical account balance snapshots.
#[utoipa::path(
    get,
    path = "/api/reports/balance-history",
    tag = "Reports",
    security(("bearer" = [])),
    params(BalanceHistoryQuery),
    responses(
        (status = 200, description = "Balance history",
         body = inline(ApiResponse<rustvault_core::models::report::BalanceHistoryReport>)),
        (status = 400, description = "Invalid query params", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn balance_history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<BalanceHistoryQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let account_ids = q.parsed_account_ids();
    let data = rustvault_core::services::report::balance_history(
        &state.pool,
        auth.user_id,
        account_ids,
        q.from,
        q.to,
    )
    .await?;
    Ok(ApiResponse::ok(data))
}

/// `GET /api/reports/cash-flow` — Monthly cash flow with 3-month forecast.
#[utoipa::path(
    get,
    path = "/api/reports/cash-flow",
    tag = "Reports",
    security(("bearer" = [])),
    params(DateRangeQuery),
    responses(
        (status = 200, description = "Cash flow report with forecast",
         body = inline(ApiResponse<rustvault_core::models::report::CashFlowReport>)),
        (status = 400, description = "Invalid query params", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn cash_flow(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<DateRangeQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let data = rustvault_core::services::report::cash_flow(&state.pool, auth.user_id, q.from, q.to)
        .await?;
    Ok(ApiResponse::ok(data))
}
