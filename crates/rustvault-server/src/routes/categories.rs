//! Category CRUD routes (hierarchical, with bulk create).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::extractors::json::ValidatedJson;
use crate::response::{ApiError, ApiResponse, ErrorBody, PaginatedResponse};
use crate::state::AppState;

use rustvault_core::models::category::{BulkCreateCategories, NewCategory, UpdateCategory};

/// `GET /api/categories` — List categories for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/categories",
    tag = "Categories",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "List of categories", body = inline(PaginatedResponse<rustvault_core::models::category::Category>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let categories = rustvault_core::services::category::list(&state.pool, auth.user_id).await?;
    Ok(PaginatedResponse::from_vec(categories))
}

/// `POST /api/categories` — Create a single category.
#[utoipa::path(
    post,
    path = "/api/categories",
    tag = "Categories",
    security(("bearer" = [])),
    request_body = NewCategory,
    responses(
        (status = 201, description = "Category created", body = inline(ApiResponse<rustvault_core::models::category::Category>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<NewCategory>,
) -> Result<impl IntoResponse, ApiError> {
    let category = rustvault_core::services::category::create(
        &state.pool,
        auth.user_id,
        &body.name,
        body.parent_id,
        body.icon.as_deref(),
        body.color.as_deref(),
        body.category_type,
    )
    .await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(category)))
}

/// `POST /api/categories/bulk` — Bulk-create categories.
#[utoipa::path(
    post,
    path = "/api/categories/bulk",
    tag = "Categories",
    security(("bearer" = [])),
    request_body = BulkCreateCategories,
    responses(
        (status = 201, description = "Categories created", body = inline(PaginatedResponse<rustvault_core::models::category::Category>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn bulk_create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<BulkCreateCategories>,
) -> Result<impl IntoResponse, ApiError> {
    let tuples: Vec<_> = body
        .categories
        .into_iter()
        .map(|c| (c.name, c.parent_id, c.icon, c.color, c.category_type))
        .collect();

    let categories =
        rustvault_core::services::category::bulk_create(&state.pool, auth.user_id, &tuples).await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(categories)))
}

/// `GET /api/categories/:id` — Get a single category.
#[utoipa::path(
    get,
    path = "/api/categories/{id}",
    tag = "Categories",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Category ID")),
    responses(
        (status = 200, description = "Category details", body = inline(ApiResponse<rustvault_core::models::category::Category>)),
        (status = 404, description = "Category not found", body = ErrorBody),
    ),
)]
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let category = rustvault_core::services::category::get(&state.pool, auth.user_id, id).await?;
    Ok(ApiResponse::ok(category))
}

/// `PUT /api/categories/:id` — Update a category.
#[utoipa::path(
    put,
    path = "/api/categories/{id}",
    tag = "Categories",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Category ID")),
    request_body = UpdateCategory,
    responses(
        (status = 200, description = "Category updated", body = inline(ApiResponse<rustvault_core::models::category::Category>)),
        (status = 404, description = "Category not found", body = ErrorBody),
    ),
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateCategory>,
) -> Result<impl IntoResponse, ApiError> {
    let category = rustvault_core::services::category::update(
        &state.pool,
        auth.user_id,
        id,
        body.name.as_deref(),
        body.parent_id,
        body.icon.as_deref(),
        body.color.as_deref(),
        body.category_type,
    )
    .await?;
    Ok(ApiResponse::ok(category))
}

/// `DELETE /api/categories/:id` — Delete a category.
#[utoipa::path(
    delete,
    path = "/api/categories/{id}",
    tag = "Categories",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Category ID")),
    responses(
        (status = 204, description = "Category deleted"),
        (status = 404, description = "Category not found", body = ErrorBody),
    ),
)]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    rustvault_core::services::category::delete(&state.pool, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
