//! Application builder — constructs the Axum router with all layers and routes.

use axum::Router;
use axum::middleware as axum_mw;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::doc::ApiDoc;
use crate::middleware::locale::locale_middleware;
use crate::routes;
use crate::state::AppState;

/// Build the complete Axum application with middleware and routes.
pub fn build_app(state: AppState) -> Router {
    routes::api_routes(state.clone())
        .merge(Scalar::with_url("/api/docs", ApiDoc::openapi()))
        // Locale resolution (innermost — runs closest to handlers).
        .layer(axum_mw::from_fn_with_state(
            state.clone(),
            locale_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
