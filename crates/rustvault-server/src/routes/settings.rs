//! Settings routes — get and update user preferences.

use axum::extract::State;
use axum::response::IntoResponse;

use crate::extractors::auth::AuthUser;
use crate::extractors::json::ValidatedJson;
use crate::response::{ApiError, ApiResponse, ErrorBody};
use crate::state::AppState;

use rustvault_core::models::settings::UpdateSettings;

/// `GET /api/settings` — Return current user settings.
#[utoipa::path(
    get,
    path = "/api/settings",
    tag = "Settings",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "User settings", body = inline(ApiResponse<rustvault_core::models::settings::UserSettings>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let settings = rustvault_core::services::settings::get(&state.pool, auth.user_id).await?;
    Ok(ApiResponse::ok(settings))
}

/// `PUT /api/settings` — Partial update of user settings.
#[utoipa::path(
    put,
    path = "/api/settings",
    tag = "Settings",
    security(("bearer" = [])),
    request_body = UpdateSettings,
    responses(
        (status = 200, description = "Settings updated", body = inline(ApiResponse<rustvault_core::models::settings::UserSettings>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<UpdateSettings>,
) -> Result<impl IntoResponse, ApiError> {
    let settings =
        rustvault_core::services::settings::update(&state.pool, auth.user_id, &body).await?;
    Ok(ApiResponse::ok(settings))
}
