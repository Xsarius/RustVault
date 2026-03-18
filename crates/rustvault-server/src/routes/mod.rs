//! API route definitions.
//!
//! Routes are split into *public* (no auth required) and *protected* (JWT
//! required) groups. The auth middleware is applied only to the protected
//! router.

pub mod accounts;
pub mod auth;
pub mod banks;
pub mod budgets;
pub mod categories;
pub mod health;
pub mod i18n;
pub mod imports;
pub mod reports;
pub mod rules;
pub mod settings;
pub mod tags;
pub mod transactions;
pub mod transfers;

use axum::Router;
use axum::middleware;
use axum::routing::{delete, get, patch, post, put};

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
        // Transactions
        .route(
            "/api/transactions",
            get(transactions::list).post(transactions::create),
        )
        .route("/api/transactions/bulk", patch(transactions::bulk_update))
        .route(
            "/api/transactions/{id}",
            get(transactions::get)
                .put(transactions::update)
                .delete(transactions::delete),
        )
        // Transfers
        .route("/api/transfers", post(transfers::create))
        .route("/api/transfers/link", post(transfers::link))
        .route("/api/transfers/detect", post(transfers::detect))
        .route("/api/transfers/{id}", delete(transfers::unlink))
        // Imports
        .route("/api/imports", get(imports::list))
        .route("/api/imports/upload", post(imports::upload))
        .route(
            "/api/imports/upload-and-execute",
            post(imports::upload_and_execute),
        )
        .route(
            "/api/imports/{id}",
            get(imports::get).delete(imports::rollback),
        )
        .route("/api/imports/{id}/preview", post(imports::preview))
        .route("/api/imports/{id}/configure", put(imports::configure))
        .route("/api/imports/{id}/execute", post(imports::execute))
        // Auto-categorization Rules
        .route("/api/rules", get(rules::list).post(rules::create))
        .route("/api/rules/test", post(rules::test_rule))
        .route("/api/rules/suggest", post(rules::suggest))
        .route(
            "/api/rules/{id}",
            get(rules::get).put(rules::update).delete(rules::delete),
        )
        // Budgets
        .route("/api/budgets", get(budgets::list).post(budgets::create))
        .route(
            "/api/budgets/{id}",
            get(budgets::get).put(budgets::update).delete(budgets::delete),
        )
        .route("/api/budgets/{id}/summary", get(budgets::summary))
        .route("/api/budgets/{id}/copy", post(budgets::copy))
        .route(
            "/api/budgets/{id}/lines",
            post(budgets::add_line),
        )
        .route("/api/budgets/{id}/lines/bulk", post(budgets::bulk_set_lines))
        .route(
            "/api/budgets/{id}/lines/{line_id}",
            put(budgets::update_line).delete(budgets::delete_line),
        )
        // Reports
        .route("/api/reports/summary", get(reports::summary))
        .route("/api/reports/income-expense", get(reports::income_expense))
        .route("/api/reports/categories/{id}/trend", get(reports::category_trend))
        .route("/api/reports/balance-history", get(reports::balance_history))
        .route("/api/reports/cash-flow", get(reports::cash_flow))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Combine public and protected routes into the full API router.
pub fn api_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(public_routes())
        .merge(protected_routes(state))
}
