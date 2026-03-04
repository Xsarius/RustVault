//! Application builder — constructs the Axum router with all layers and routes.

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::routes;

/// Build the complete Axum application with middleware and routes.
pub async fn build_app() -> anyhow::Result<Router> {
    let app = Router::new()
        .merge(routes::api_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    Ok(app)
}
