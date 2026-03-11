//! Shared application state.

use sqlx::PgPool;
use std::sync::Arc;

use crate::config::AppConfig;
use rustvault_core::i18n::I18n;

/// Shared state available to all route handlers via Axum's [`axum::extract::State`].
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool.
    pub pool: PgPool,
    /// Application configuration.
    pub config: Arc<AppConfig>,
    /// Internationalization context (loaded Fluent bundles).
    pub i18n: I18n,
}

impl AppState {
    /// Create a new `AppState`.
    pub fn new(pool: PgPool, config: AppConfig, i18n: I18n) -> Self {
        Self {
            pool,
            config: Arc::new(config),
            i18n,
        }
    }
}
