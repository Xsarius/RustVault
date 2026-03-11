//! i18n routes — list available locales.

use axum::extract::State;
use axum::response::IntoResponse;

use crate::response::ApiResponse;
use crate::state::AppState;

/// `GET /api/i18n/locales` — List all available locales with completeness info.
///
/// **Auth:** None (public endpoint).
///
/// Returns locale metadata including BCP 47 code, English name,
/// native name, translation completeness percentage, and whether
/// the locale is the instance default.
#[utoipa::path(
    get,
    path = "/api/i18n/locales",
    tag = "i18n",
    responses(
        (status = 200, description = "Available locales", body = inline(Vec<rustvault_core::i18n::LocaleInfo>)),
    ),
)]
pub async fn list_locales(State(state): State<AppState>) -> impl IntoResponse {
    let locales = state.i18n.available_locales().to_vec();
    ApiResponse::ok(locales)
}
