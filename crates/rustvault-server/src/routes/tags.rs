//! Tag CRUD routes (with bulk create).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::extractors::json::ValidatedJson;
use crate::response::{ApiError, ApiResponse, ErrorBody, PaginatedResponse};
use crate::state::AppState;

use rustvault_core::models::tag::{BulkCreateTags, NewTag, UpdateTag};

/// `GET /api/tags` — List tags for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/tags",
    tag = "Tags",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "List of tags", body = inline(PaginatedResponse<rustvault_core::models::tag::Tag>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let tags = rustvault_core::services::tag::list(&state.pool, auth.user_id).await?;
    Ok(PaginatedResponse::from_vec(tags))
}

/// `POST /api/tags` — Create a single tag.
#[utoipa::path(
    post,
    path = "/api/tags",
    tag = "Tags",
    security(("bearer" = [])),
    request_body = NewTag,
    responses(
        (status = 201, description = "Tag created", body = inline(ApiResponse<rustvault_core::models::tag::Tag>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<NewTag>,
) -> Result<impl IntoResponse, ApiError> {
    let tag = rustvault_core::services::tag::create(
        &state.pool,
        auth.user_id,
        &body.name,
        body.color.as_deref(),
    )
    .await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(tag)))
}

/// `POST /api/tags/bulk` — Bulk-create tags.
#[utoipa::path(
    post,
    path = "/api/tags/bulk",
    tag = "Tags",
    security(("bearer" = [])),
    request_body = BulkCreateTags,
    responses(
        (status = 201, description = "Tags created", body = inline(PaginatedResponse<rustvault_core::models::tag::Tag>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn bulk_create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<BulkCreateTags>,
) -> Result<impl IntoResponse, ApiError> {
    let tuples: Vec<_> = body
        .tags
        .into_iter()
        .map(|t| (t.name, t.color))
        .collect();

    let tags =
        rustvault_core::services::tag::bulk_create(&state.pool, auth.user_id, &tuples).await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(tags)))
}

/// `GET /api/tags/:id` — Get a single tag.
#[utoipa::path(
    get,
    path = "/api/tags/{id}",
    tag = "Tags",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Tag ID")),
    responses(
        (status = 200, description = "Tag details", body = inline(ApiResponse<rustvault_core::models::tag::Tag>)),
        (status = 404, description = "Tag not found", body = ErrorBody),
    ),
)]
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let tag = rustvault_core::services::tag::get(&state.pool, auth.user_id, id).await?;
    Ok(ApiResponse::ok(tag))
}

/// `PUT /api/tags/:id` — Update a tag.
#[utoipa::path(
    put,
    path = "/api/tags/{id}",
    tag = "Tags",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Tag ID")),
    request_body = UpdateTag,
    responses(
        (status = 200, description = "Tag updated", body = inline(ApiResponse<rustvault_core::models::tag::Tag>)),
        (status = 404, description = "Tag not found", body = ErrorBody),
    ),
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateTag>,
) -> Result<impl IntoResponse, ApiError> {
    let tag = rustvault_core::services::tag::update(
        &state.pool,
        auth.user_id,
        id,
        body.name.as_deref(),
        body.color.as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(tag))
}

/// `DELETE /api/tags/:id` — Delete a tag.
#[utoipa::path(
    delete,
    path = "/api/tags/{id}",
    tag = "Tags",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Tag ID")),
    responses(
        (status = 204, description = "Tag deleted"),
        (status = 404, description = "Tag not found", body = ErrorBody),
    ),
)]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    rustvault_core::services::tag::delete(&state.pool, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
