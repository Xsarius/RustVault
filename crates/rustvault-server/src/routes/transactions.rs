//! Transaction CRUD routes.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::extractors::json::ValidatedJson;
use crate::response::{ApiError, ApiResponse, ErrorBody, PaginatedResponse};
use crate::state::AppState;

use rustvault_core::models::transaction::{
    BulkUpdateTransactions, NewTransaction, TransactionListQuery, UpdateTransaction,
};

/// Cursor encoding: `date|uuid`.
fn decode_cursor(cursor: &str) -> Option<(time::Date, Uuid)> {
    let parts: Vec<&str> = cursor.split('|').collect();
    if parts.len() != 2 {
        return None;
    }
    let date = time::Date::parse(
        parts[0],
        time::macros::format_description!("[year]-[month]-[day]"),
    )
    .ok()?;
    let id = Uuid::parse_str(parts[1]).ok()?;
    Some((date, id))
}

fn encode_cursor(date: time::Date, id: Uuid) -> String {
    format!(
        "{}|{}",
        date.format(time::macros::format_description!("[year]-[month]-[day]"))
            .unwrap_or_default(),
        id
    )
}

/// `GET /api/transactions` — List transactions with filters and pagination.
#[utoipa::path(
    get,
    path = "/api/transactions",
    tag = "Transactions",
    security(("bearer" = [])),
    params(TransactionListQuery),
    responses(
        (status = 200, description = "List of transactions", body = inline(PaginatedResponse<rustvault_core::models::transaction::Transaction>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<TransactionListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.clamp(1, 100);
    let (cursor_date, cursor_id) = query
        .cursor
        .as_deref()
        .and_then(decode_cursor)
        .map(|(d, id)| (Some(d), Some(id)))
        .unwrap_or((None, None));

    let transactions = rustvault_core::services::transaction::list(
        &state.pool,
        auth.user_id,
        query.account_id,
        query.category_id,
        query.transaction_type.as_deref(),
        query.date_from,
        query.date_to,
        query.q.as_deref(),
        query.is_reviewed,
        query.tag_id,
        query.import_id,
        limit + 1, // fetch one extra to determine has_more
        cursor_date,
        cursor_id,
    )
    .await?;

    let has_more = transactions.len() as i64 > limit;
    let items: Vec<_> = transactions.into_iter().take(limit as usize).collect();
    let next_cursor = if has_more {
        items.last().map(|t| encode_cursor(t.date, t.id))
    } else {
        None
    };

    Ok(PaginatedResponse::build(items, has_more, next_cursor))
}

/// `POST /api/transactions` — Create a new transaction.
#[utoipa::path(
    post,
    path = "/api/transactions",
    tag = "Transactions",
    security(("bearer" = [])),
    request_body = NewTransaction,
    responses(
        (status = 201, description = "Transaction created", body = inline(ApiResponse<rustvault_core::models::transaction::Transaction>)),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<NewTransaction>,
) -> Result<impl IntoResponse, ApiError> {
    let transaction = rustvault_core::services::transaction::create(
        &state.pool,
        auth.user_id,
        body.account_id,
        body.category_id,
        body.transaction_type,
        body.amount,
        body.date,
        &body.description,
        body.payee.as_deref(),
        body.notes.as_deref(),
        &body.tag_ids,
    )
    .await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(transaction)))
}

/// `GET /api/transactions/:id` — Get a single transaction.
#[utoipa::path(
    get,
    path = "/api/transactions/{id}",
    tag = "Transactions",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Transaction ID")),
    responses(
        (status = 200, description = "Transaction details", body = inline(ApiResponse<rustvault_core::models::transaction::Transaction>)),
        (status = 404, description = "Not found", body = ErrorBody),
    ),
)]
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let transaction =
        rustvault_core::services::transaction::get(&state.pool, auth.user_id, id).await?;
    Ok(ApiResponse::ok(transaction))
}

/// `PUT /api/transactions/:id` — Update a transaction.
#[utoipa::path(
    put,
    path = "/api/transactions/{id}",
    tag = "Transactions",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Transaction ID")),
    request_body = UpdateTransaction,
    responses(
        (status = 200, description = "Transaction updated", body = inline(ApiResponse<rustvault_core::models::transaction::Transaction>)),
        (status = 404, description = "Not found", body = ErrorBody),
    ),
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateTransaction>,
) -> Result<impl IntoResponse, ApiError> {
    let payee_ref = body
        .payee
        .as_ref()
        .map(|opt| opt.as_deref());
    let notes_ref = body
        .notes
        .as_ref()
        .map(|opt| opt.as_deref());

    let transaction = rustvault_core::services::transaction::update(
        &state.pool,
        auth.user_id,
        id,
        body.category_id,
        body.transaction_type,
        body.amount,
        body.date,
        body.description.as_deref(),
        payee_ref,
        notes_ref,
        body.is_reviewed,
        body.tag_ids.as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(transaction))
}

/// `DELETE /api/transactions/:id` — Soft-delete a transaction.
#[utoipa::path(
    delete,
    path = "/api/transactions/{id}",
    tag = "Transactions",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Transaction ID")),
    responses(
        (status = 204, description = "Transaction deleted"),
        (status = 404, description = "Not found", body = ErrorBody),
    ),
)]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    rustvault_core::services::transaction::delete(&state.pool, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `PATCH /api/transactions/bulk` — Bulk update transactions.
#[utoipa::path(
    patch,
    path = "/api/transactions/bulk",
    tag = "Transactions",
    security(("bearer" = [])),
    request_body = BulkUpdateTransactions,
    responses(
        (status = 200, description = "Bulk update result", body = inline(ApiResponse<serde_json::Value>)),
        (status = 400, description = "Validation error", body = ErrorBody),
    ),
)]
pub async fn bulk_update(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<BulkUpdateTransactions>,
) -> Result<impl IntoResponse, ApiError> {
    let updated = rustvault_core::services::transaction::bulk_update(
        &state.pool,
        auth.user_id,
        &body.transaction_ids,
        body.category_id,
        body.is_reviewed,
        &body.add_tag_ids,
    )
    .await?;

    Ok(ApiResponse::ok(serde_json::json!({ "updated": updated })))
}
