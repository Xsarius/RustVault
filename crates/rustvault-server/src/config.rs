//! Application configuration.
//!
//! Loading priority: env vars (secrets) > config.toml (tuning) > defaults.
//! Secrets (JWT keys, OIDC credentials, DB password) are **always** loaded from
//! environment variables and never stored in config files.

use serde::Deserialize;
use std::path::Path;
use tracing::info;

/// Top-level application configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// HTTP server settings.
    pub server: ServerConfig,
    /// Database connection settings.
    pub database: DatabaseConfig,
    /// Authentication settings.
    pub auth: AuthConfig,
    /// Import pipeline settings.
    pub import: ImportConfig,
    /// AI feature settings.
    pub ai: AiConfig,
    // --- Secrets (env vars only) ---
    /// JWT signing secret (minimum 256-bit).
    pub jwt_secret: String,
    /// Previous JWT secret for graceful key rotation.
    pub jwt_secret_old: Option<String>,
    /// AES-256 encryption key for sensitive data (hex-encoded 32 bytes).
    pub encryption_key: Option<String>,
    // --- OIDC secrets (env vars only) ---
    /// OIDC client ID.
    pub oidc_client_id: Option<String>,
    /// OIDC client secret.
    pub oidc_client_secret: Option<String>,
    /// OIDC issuer URL (e.g., <https://auth.example.com/application/o/rustvault/>).
    pub oidc_issuer_url: Option<String>,
}

/// HTTP server configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Port to listen on.
    pub port: u16,
    /// Allowed CORS origins.
    pub allowed_origins: Vec<String>,
    /// Request timeout in seconds.
    pub request_timeout_secs: u64,
    /// Maximum JSON body size (e.g., "10MB").
    pub max_body_size: String,
    /// Maximum file upload size (e.g., "50MB").
    pub max_upload_size: String,
    /// Path to the locales directory (relative to CWD or absolute).
    pub locales_dir: String,
    /// Path to the built frontend static assets directory.
    ///
    /// Axum serves this directory at `/` with a SPA fallback to `index.html`.
    /// Set to an empty string to disable static file serving (API-only mode).
    pub static_dir: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            allowed_origins: Vec::new(),
            request_timeout_secs: 30,
            max_body_size: "10MB".to_string(),
            max_upload_size: "50MB".to_string(),
            locales_dir: "locales".to_string(),
            static_dir: "static".to_string(),
        }
    }
}

/// Database configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    /// Fully-constructed connection URL (overrides host/port/user/password/name).
    #[serde(default)]
    pub url: String,
    /// Maximum number of connections in the pool.
    pub max_connections: u32,
    /// Minimum number of idle connections.
    pub min_connections: u32,
    /// Connection acquire timeout in seconds.
    pub acquire_timeout_secs: u64,
    /// Idle connection timeout in seconds.
    pub idle_timeout_secs: u64,
    /// Maximum connection lifetime in seconds.
    pub max_lifetime_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10,
            min_connections: 2,
            acquire_timeout_secs: 5,
            idle_timeout_secs: 300,
            max_lifetime_secs: 1800,
        }
    }
}

/// Authentication configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    /// Allow public user registration.
    pub allow_new_user_register: bool,
    /// Access token TTL in seconds.
    pub access_token_ttl_secs: u64,
    /// Refresh token TTL in seconds.
    pub refresh_token_ttl_secs: u64,
    /// Maximum active sessions per user.
    pub max_sessions_per_user: u32,
    /// Minimum password length.
    pub password_min_length: usize,
    /// Maximum password length.
    pub password_max_length: usize,
    /// Login rate limit: max attempts.
    pub login_rate_limit_attempts: u32,
    /// Login rate limit: window in seconds.
    pub login_rate_limit_window_secs: u64,
    /// Account lockout after N failed attempts.
    pub account_lockout_threshold: u32,
    /// Audit log retention in days.
    pub audit_retention_days: u32,
    /// OIDC configuration.
    pub oidc: OidcConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            allow_new_user_register: true,
            access_token_ttl_secs: 900,
            refresh_token_ttl_secs: 604_800,
            max_sessions_per_user: 10,
            password_min_length: 10,
            password_max_length: 128,
            login_rate_limit_attempts: 5,
            login_rate_limit_window_secs: 900,
            account_lockout_threshold: 20,
            audit_retention_days: 90,
            oidc: OidcConfig::default(),
        }
    }
}

/// OIDC / SSO configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OidcConfig {
    /// Whether OIDC is enabled.
    pub enabled: bool,
    /// Display name for the OIDC button (e.g., "Sign in with Authentik").
    pub display_name: String,
    /// OIDC scopes to request.
    pub scopes: Vec<String>,
    /// Auto-create local user on first OIDC login.
    pub auto_register: bool,
}

impl Default for OidcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            display_name: "SSO".to_string(),
            scopes: vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
            ],
            auto_register: true,
        }
    }
}

/// Import pipeline configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ImportConfig {
    /// Maximum upload file size (e.g., "50MB").
    pub max_file_size: String,
    /// Allowed file extensions for import.
    pub allowed_extensions: Vec<String>,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            max_file_size: "50MB".to_string(),
            allowed_extensions: vec![
                "csv".into(),
                "mt940".into(),
                "sta".into(),
                "ofx".into(),
                "qfx".into(),
                "qif".into(),
                "xml".into(),
                "xlsx".into(),
                "xls".into(),
                "ods".into(),
                "json".into(),
                "pdf".into(),
            ],
        }
    }
}

/// AI feature configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AiConfig {
    /// Master toggle — false disables all AI features.
    pub enabled: bool,
}

/// Raw TOML file structure (deserialization target).
#[derive(Debug, Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    server: ServerConfig,
    #[serde(default)]
    database: DatabaseConfig,
    #[serde(default)]
    auth: AuthConfig,
    #[serde(default)]
    import: ImportConfig,
    #[serde(default)]
    ai: AiConfig,
}

/// Configuration loading error.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Failed to read config file.
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    /// Failed to parse TOML.
    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
    /// Missing required environment variable.
    #[error("missing required env var: {0}")]
    MissingEnv(String),
}

impl AppConfig {
    /// Load configuration from env vars + optional TOML file.
    ///
    /// 1. Read `RUSTVAULT_CONFIG` env var or fall back to `config.toml`.
    /// 2. Parse TOML into config structs (all fields have defaults).
    /// 3. Overlay secret env vars (JWT_SECRET, DATABASE_*, OIDC_*).
    /// 4. Construct database URL from individual env vars if `database.url` is empty.
    pub fn load() -> Result<Self, ConfigError> {
        let config_path =
            std::env::var("RUSTVAULT_CONFIG").unwrap_or_else(|_| "config.toml".to_string());

        let raw: RawConfig = if Path::new(&config_path).exists() {
            info!(path = %config_path, "Loading config from file");
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            info!("No config file found, using defaults");
            RawConfig::default()
        };

        // Construct database URL from env vars if not set in config
        let database_url = if raw.database.url.is_empty() {
            Self::build_database_url()?
        } else {
            raw.database.url.clone()
        };

        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnv("JWT_SECRET".to_string()))?;

        let jwt_secret_old = std::env::var("JWT_SECRET_OLD").ok();
        let encryption_key = std::env::var("ENCRYPTION_KEY").ok();
        let oidc_client_id = std::env::var("OIDC_CLIENT_ID").ok();
        let oidc_client_secret = std::env::var("OIDC_CLIENT_SECRET").ok();
        let oidc_issuer_url = std::env::var("OIDC_ISSUER_URL").ok();

        Ok(Self {
            server: raw.server,
            database: DatabaseConfig {
                url: database_url,
                ..raw.database
            },
            auth: raw.auth,
            import: raw.import,
            ai: raw.ai,
            jwt_secret,
            jwt_secret_old,
            encryption_key,
            oidc_client_id,
            oidc_client_secret,
            oidc_issuer_url,
        })
    }

    /// Build a PostgreSQL connection URL from individual `DATABASE_*` env vars.
    fn build_database_url() -> Result<String, ConfigError> {
        let host = std::env::var("DATABASE_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("DATABASE_PORT").unwrap_or_else(|_| "5432".to_string());
        let user = std::env::var("DATABASE_USER").unwrap_or_else(|_| "rustvault".to_string());
        let password = std::env::var("DATABASE_PASSWORD")
            .map_err(|_| ConfigError::MissingEnv("DATABASE_PASSWORD".to_string()))?;
        let name = std::env::var("DATABASE_NAME").unwrap_or_else(|_| "rustvault".to_string());

        Ok(format!("postgres://{user}:{password}@{host}:{port}/{name}"))
    }

    /// Parse a size string like "10MB" into bytes.
    pub fn parse_size(size: &str) -> usize {
        let size = size.trim().to_uppercase();
        if let Some(mb) = size.strip_suffix("MB") {
            mb.trim().parse::<usize>().unwrap_or(10) * 1024 * 1024
        } else if let Some(kb) = size.strip_suffix("KB") {
            kb.trim().parse::<usize>().unwrap_or(1024) * 1024
        } else if let Some(gb) = size.strip_suffix("GB") {
            gb.trim().parse::<usize>().unwrap_or(1) * 1024 * 1024 * 1024
        } else {
            size.parse::<usize>().unwrap_or(10 * 1024 * 1024)
        }
    }
}
