//! Authentication routes: register, login, refresh, me.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;

use crate::extractors::auth::AuthUser;
use crate::extractors::json::ValidatedJson;
use crate::response::{ApiError, ApiResponse, ErrorBody};
use crate::state::AppState;

use rustvault_core::models::user::{LoginRequest, NewUser};

/// `POST /api/auth/register` — Create a new user account.
#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "Auth",
    request_body = NewUser,
    responses(
        (status = 201, description = "User created", body = inline(ApiResponse<rustvault_core::models::user::UserInfo>)),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 409, description = "Email already registered", body = ErrorBody),
    ),
)]
pub async fn register(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<NewUser>,
) -> Result<impl IntoResponse, ApiError> {
    let user_info = rustvault_core::services::auth::register(
        &state.pool,
        &body.username,
        &body.email,
        &body.password,
        state.config.auth.allow_new_user_register,
    )
    .await?;

    Ok((StatusCode::CREATED, ApiResponse::ok(user_info)))
}

/// `POST /api/auth/login` — Authenticate and return tokens.
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful — returns access and refresh tokens"),
        (status = 401, description = "Invalid credentials", body = ErrorBody),
    ),
)]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let (tokens, refresh_token) = rustvault_core::services::auth::login(
        &state.pool,
        &body.email,
        &body.password,
        &state.config.jwt_secret,
        state.config.auth.access_token_ttl_secs,
        state.config.auth.refresh_token_ttl_secs,
        user_agent.as_deref(),
        None, // IP address extraction would require a real extractor
    )
    .await?;

    // In production, the refresh token would be set as an HttpOnly cookie.
    // For now, include it in the response for API testing.
    let response = serde_json::json!({
        "data": {
            "access_token": tokens.access_token,
            "token_type": tokens.token_type,
            "expires_in": tokens.expires_in,
            "refresh_token": refresh_token,
        }
    });

    Ok(Json(response))
}

/// `POST /api/auth/refresh` — Refresh access token.
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct RefreshRequest {
    /// The refresh token to exchange.
    pub refresh_token: String,
}

/// `POST /api/auth/refresh` — Rotate access + refresh tokens.
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "Auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Tokens refreshed — returns new access and refresh tokens"),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorBody),
    ),
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let (tokens, new_refresh) = rustvault_core::services::auth::refresh(
        &state.pool,
        &body.refresh_token,
        &state.config.jwt_secret,
        state.config.auth.access_token_ttl_secs,
        state.config.auth.refresh_token_ttl_secs,
    )
    .await?;

    let response = serde_json::json!({
        "data": {
            "access_token": tokens.access_token,
            "token_type": tokens.token_type,
            "expires_in": tokens.expires_in,
            "refresh_token": new_refresh,
        }
    });

    Ok(Json(response))
}

/// `GET /api/auth/me` — Get current user info.
#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "Auth",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "Current user profile", body = inline(ApiResponse<rustvault_core::models::user::UserInfo>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let user_info = rustvault_core::services::auth::me(&state.pool, auth.user_id).await?;
    Ok(ApiResponse::ok(user_info))
}

// ── OIDC Routes ──────────────────────────────────────────────

/// `GET /api/auth/oidc/authorize` — Start OIDC login flow.
///
/// Returns the authorization URL that the client should redirect to.
/// The `state` and `nonce` must be stored client-side (e.g., in a session cookie)
/// and sent back in the callback for CSRF/replay protection.
pub async fn oidc_authorize(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let config = &state.config;

    if !config.auth.oidc.enabled {
        return Err(ApiError::from(rustvault_core::CoreError::OidcNotConfigured));
    }

    let issuer_url = config
        .oidc_issuer_url
        .as_deref()
        .ok_or_else(|| ApiError::from(rustvault_core::CoreError::OidcNotConfigured))?;
    let client_id = config
        .oidc_client_id
        .as_deref()
        .ok_or_else(|| ApiError::from(rustvault_core::CoreError::OidcNotConfigured))?;
    let client_secret = config
        .oidc_client_secret
        .as_deref()
        .ok_or_else(|| ApiError::from(rustvault_core::CoreError::OidcNotConfigured))?;

    let redirect_url = format!(
        "{}://{}:{}/api/auth/oidc/callback",
        "http", "localhost", config.server.port
    );

    let (authorize_url, csrf_state, nonce) =
        rustvault_core::services::auth::oidc_authorize_url(
            issuer_url,
            client_id,
            client_secret,
            &redirect_url,
            &config.auth.oidc.scopes,
        )
        .await?;

    let response = serde_json::json!({
        "data": {
            "authorize_url": authorize_url,
            "state": csrf_state,
            "nonce": nonce,
        }
    });

    Ok(Json(response))
}

/// `POST /api/auth/oidc/callback` — Complete OIDC login flow.
///
/// The client sends the `code` from the OIDC provider redirect,
/// along with the `nonce` it stored during the authorize step.
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct OidcCallbackBody {
    /// Authorization code from the OIDC provider.
    pub code: String,
    /// Nonce from the authorize step (stored client-side).
    pub nonce: String,
}

/// `POST /api/auth/oidc/callback` — Handle OIDC provider callback.
pub async fn oidc_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<OidcCallbackBody>,
) -> Result<impl IntoResponse, ApiError> {
    let config = &state.config;

    if !config.auth.oidc.enabled {
        return Err(ApiError::from(rustvault_core::CoreError::OidcNotConfigured));
    }

    let issuer_url = config
        .oidc_issuer_url
        .as_deref()
        .ok_or_else(|| ApiError::from(rustvault_core::CoreError::OidcNotConfigured))?;
    let client_id = config
        .oidc_client_id
        .as_deref()
        .ok_or_else(|| ApiError::from(rustvault_core::CoreError::OidcNotConfigured))?;
    let client_secret = config
        .oidc_client_secret
        .as_deref()
        .ok_or_else(|| ApiError::from(rustvault_core::CoreError::OidcNotConfigured))?;

    let redirect_url = format!(
        "{}://{}:{}/api/auth/oidc/callback",
        "http", "localhost", config.server.port
    );

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let (tokens, refresh_token) = rustvault_core::services::auth::oidc_callback(
        &state.pool,
        issuer_url,
        client_id,
        client_secret,
        &redirect_url,
        &body.code,
        &body.nonce,
        config.auth.oidc.auto_register,
        &config.jwt_secret,
        config.auth.access_token_ttl_secs,
        config.auth.refresh_token_ttl_secs,
        user_agent.as_deref(),
        None,
    )
    .await?;

    let response = serde_json::json!({
        "data": {
            "access_token": tokens.access_token,
            "token_type": tokens.token_type,
            "expires_in": tokens.expires_in,
            "refresh_token": refresh_token,
        }
    });

    Ok(Json(response))
}
