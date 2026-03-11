//! Resolved locale extractor.
//!
//! Extracts the user's resolved locale from request extensions
//! (set by the locale middleware). Falls back to the instance
//! default locale if the middleware has not run.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::state::AppState;

/// The resolved locale for the current request.
///
/// Set by [`crate::middleware::locale::locale_middleware`] based on the
/// `Accept-Language` header. Handlers can extract this to format
/// localized responses.
///
/// # Example
///
/// ```ignore
/// async fn handler(locale: ResolvedLocale) -> impl IntoResponse {
///     // locale.0 is e.g. "en-US"
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ResolvedLocale(pub String);

impl FromRequestParts<AppState> for ResolvedLocale {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // If the locale middleware ran, use its result.
        if let Some(locale) = parts.extensions.get::<ResolvedLocale>() {
            return Ok(locale.clone());
        }

        // Fallback: resolve from Accept-Language header directly.
        let accept_language = parts
            .headers
            .get("accept-language")
            .and_then(|v| v.to_str().ok());
        Ok(ResolvedLocale(state.i18n.resolve_locale(accept_language)))
    }
}
