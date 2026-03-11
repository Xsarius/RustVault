//! RustVault HTTP server.
//!
//! Binary crate that initializes the Axum web server, loads configuration,
//! sets up tracing, and wires together all domain modules.

use tracing::info;
use tracing_subscriber::EnvFilter;

use rustvault_server::config::AppConfig;
use rustvault_server::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file (ignore if missing)
    let _ = dotenvy::dotenv();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting RustVault server");

    // Load configuration (TOML + env vars)
    let config = AppConfig::load()?;
    let port = config.server.port;

    // Create database connection pool and run migrations
    let pool = rustvault_db::create_pool(
        &config.database.url,
        config.database.max_connections,
    )
    .await?;

    // Load i18n locale bundles
    let i18n = rustvault_core::i18n::I18n::load(std::path::Path::new(
        &config.server.locales_dir,
    ))
    .map_err(|e| anyhow::anyhow!("failed to load i18n: {e}"))?;

    // Build application state and router
    let state = AppState::new(pool, config, i18n);
    let app = rustvault_server::app::build_app(state);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
