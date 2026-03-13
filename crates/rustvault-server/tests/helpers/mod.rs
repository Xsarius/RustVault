//! Shared test helpers for integration tests.
//!
//! Provides [`TestApp`] — a wrapper around the Axum app with an ephemeral
//! database, pre-configured JWT secret, and convenience methods for
//! creating users and making authenticated requests.

use axum::Router;
use serde_json::Value;
use sqlx::PgPool;

/// JWT secret used in all tests.
pub const TEST_JWT_SECRET: &str = "test-jwt-secret-that-is-long-enough-for-hs256-validation";

/// Default test user credentials.
pub const TEST_USER: &str = "testuser";
pub const TEST_EMAIL: &str = "test@example.com";
pub const TEST_PASSWORD: &str = "securepassword123";

/// Build the Axum app with the given pool and default test config.
pub fn build_test_app(pool: PgPool) -> Router {
    use rustvault_core::i18n::I18n;

    let config = rustvault_server::config::AppConfig {
        server: rustvault_server::config::ServerConfig {
            port: 0,
            allowed_origins: vec![],
            request_timeout_secs: 30,
            max_body_size: "10MB".into(),
            max_upload_size: "50MB".into(),
            locales_dir: workspace_locales_dir(),
            static_dir: String::new(),
        },
        database: rustvault_server::config::DatabaseConfig::default(),
        auth: rustvault_server::config::AuthConfig::default(),
        import: rustvault_server::config::ImportConfig::default(),
        ai: rustvault_server::config::AiConfig::default(),
        jwt_secret: TEST_JWT_SECRET.into(),
        jwt_secret_old: None,
        encryption_key: None,
        oidc_client_id: None,
        oidc_client_secret: None,
        oidc_issuer_url: None,
    };

    let i18n =
        I18n::load(std::path::Path::new(&config.server.locales_dir)).expect("load i18n bundles");

    let state = rustvault_server::state::AppState::new(pool, config, i18n);
    rustvault_server::app::build_app(state)
}

/// Resolve the workspace-root `locales/` directory from the test binary location.
fn workspace_locales_dir() -> String {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent() // crates/
        .and_then(|p| p.parent()) // workspace root
        .expect("workspace root")
        .join("locales")
        .to_str()
        .expect("utf-8 path")
        .to_string()
}

/// Register a test user and return the access token + refresh token.
pub async fn register_and_login(app: &axum_test::TestServer) -> (String, String) {
    // Register
    app.post("/api/auth/register")
        .json(&serde_json::json!({
            "username": TEST_USER,
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
        }))
        .await;

    // Login
    let res = app
        .post("/api/auth/login")
        .json(&serde_json::json!({
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
        }))
        .await;

    let body: Value = res.json();
    let access = body["data"]["access_token"]
        .as_str()
        .expect("access_token")
        .to_string();
    let refresh = body["data"]["refresh_token"]
        .as_str()
        .expect("refresh_token")
        .to_string();

    (access, refresh)
}

/// Create a [`axum_test::TestServer`] from a pool (call inside `#[sqlx::test]`).
pub fn test_server(pool: PgPool) -> axum_test::TestServer {
    let app = build_test_app(pool);
    axum_test::TestServer::new(app)
}
