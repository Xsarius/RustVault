//! API route definitions.
//!
//! Routes are split into *public* (no auth required) and *protected* (JWT
//! required) groups. The auth middleware is applied only to the protected
//! router.

pub mod accounts;
pub mod auth;
pub mod banks;
pub mod categories;
pub mod health;
pub mod i18n;
pub mod settings;
pub mod tags;

use axum::middleware;
use axum::routing::{get, post, put};
use axum::Router;

use crate::middleware::auth::auth_middleware;
use crate::state::AppState;

/// Routes that do **not** require authentication.
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/refresh", post(auth::refresh))
        // OIDC
        .route("/api/auth/oidc/authorize", get(auth::oidc_authorize))
        .route("/api/auth/oidc/callback", post(auth::oidc_callback))
        // i18n
        .route("/api/i18n/locales", get(i18n::list_locales))
}

/// Routes that require a valid JWT access token.
fn protected_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Auth
        .route("/api/auth/me", get(auth::me))
        // Banks
        .route("/api/banks", get(banks::list).post(banks::create))
        .route("/api/banks/{id}", get(banks::get).put(banks::update))
        .route("/api/banks/{id}/archive", put(banks::archive))
        // Accounts
        .route(
            "/api/accounts",
            get(accounts::list).post(accounts::create),
        )
        .route(
            "/api/accounts/{id}",
            get(accounts::get).put(accounts::update),
        )
        .route("/api/accounts/{id}/archive", put(accounts::archive))
        // Categories
        .route(
            "/api/categories",
            get(categories::list).post(categories::create),
        )
        .route("/api/categories/bulk", post(categories::bulk_create))
        .route(
            "/api/categories/{id}",
            get(categories::get)
                .put(categories::update)
                .delete(categories::delete),
        )
        // Tags
        .route("/api/tags", get(tags::list).post(tags::create))
        .route("/api/tags/bulk", post(tags::bulk_create))
        .route(
            "/api/tags/{id}",
            get(tags::get).put(tags::update).delete(tags::delete),
        )
        // Settings
        .route(
            "/api/settings",
            get(settings::get).put(settings::update),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Combine public and protected routes into the full API router.
pub fn api_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(public_routes())
        .merge(protected_routes(state))
}
