//! RustVault HTTP server.
//!
//! Binary crate that initializes the Axum web server, loads configuration,
//! sets up tracing, and wires together all domain modules.

#![warn(missing_docs)]

mod app;
mod routes;

use tracing::info;
use tracing_subscriber::EnvFilter;

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

    let app = app::build_app().await?;

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
