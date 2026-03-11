//! Locale resolution middleware.
//!
//! Parses the `Accept-Language` header and resolves the best matching locale
//! from the available Fluent bundles. Stores a [`ResolvedLocale`] in request
//! extensions for downstream extractors and handlers.

use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::extractors::locale::ResolvedLocale;
use crate::state::AppState;

/// Axum middleware that resolves the user's locale from the `Accept-Language` header.
///
/// Applied to all routes (both public and protected). The resolved locale
/// is stored in request extensions and can be extracted via [`ResolvedLocale`].
pub async fn locale_middleware(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let accept_language = req
        .headers()
        .get("accept-language")
        .and_then(|v| v.to_str().ok());

    let locale = state.i18n.resolve_locale(accept_language);
    req.extensions_mut().insert(ResolvedLocale(locale));

    next.run(req).await
}
